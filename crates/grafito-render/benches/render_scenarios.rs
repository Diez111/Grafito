#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(deprecated)]
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use grafito_core::{
    CircleObj, Document, FunctionObj, GeoObject, LineObj, ParametricCurve3DObj, PointObj,
    PolygonObj, Surface3DObj, VectorField2DObj,
};
use grafito_geometry::{Camera3D, Point2, ViewTransform};
use grafito_render::Renderer;

fn view_800x600() -> ViewTransform {
    ViewTransform::new(800.0, 600.0)
}

fn make_doc_with_mixed_2d_objects(count: usize) -> Document {
    let mut doc = Document::new();
    for i in 0..count {
        let obj = match i % 5 {
            0 => GeoObject::Point(PointObj::new(Point2::new(
                (i as f64) * 0.5,
                (i as f64).sin(),
            ))),
            1 => GeoObject::Line(LineObj::new(
                Point2::new(i as f64, 0.0),
                Point2::new(i as f64 + 1.0, 1.0),
            )),
            2 => GeoObject::Circle(CircleObj::new(
                Point2::new((i as f64) * 0.2, (i as f64) * 0.1),
                0.5,
            )),
            3 => GeoObject::Polygon(PolygonObj::new(vec![
                Point2::new(0.0, 0.0),
                Point2::new(1.0, 0.0),
                Point2::new(1.0, 1.0),
                Point2::new(0.0, 1.0),
            ])),
            _ => GeoObject::Function(FunctionObj::new("sin(x)")),
        };
        doc.add_object(obj);
    }
    doc
}

fn bench_build_geometry_many_objects(c: &mut Criterion) {
    let doc = make_doc_with_mixed_2d_objects(100);
    let view = view_800x600();

    c.bench_function("build_geometry_many_objects", |b| {
        b.iter(|| {
            let (vertices, indices) =
                Renderer::build_geometry_static(black_box(&doc), black_box(&view), false, true);
            black_box((vertices.len(), indices.len()));
        })
    });
}

fn bench_build_geometry_with_functions(c: &mut Criterion) {
    let mut doc = Document::new();
    for i in 0..10 {
        doc.add_object(GeoObject::Function(FunctionObj::new(format!(
            "sin({}*x)",
            i + 1
        ))));
    }
    let view = view_800x600();

    c.bench_function("build_geometry_with_functions", |b| {
        b.iter(|| {
            let (vertices, indices) =
                Renderer::build_geometry_static(black_box(&doc), black_box(&view), false, true);
            black_box((vertices.len(), indices.len()));
        })
    });
}

fn try_headless_renderer() -> Option<std::sync::Arc<Renderer>> {
    use std::sync::{Arc, OnceLock};
    static RENDERER: OnceLock<Option<Arc<Renderer>>> = OnceLock::new();
    RENDERER
        .get_or_init(|| {
            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
            let adapter = instance
                .enumerate_adapters(wgpu::Backends::all())
                .into_iter()
                .next()?;
            let (device, queue) = pollster::block_on(adapter.request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_defaults(),
                    label: None,
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            ))
            .ok()?;
            Some(Arc::new(Renderer::new(
                &device,
                &queue,
                wgpu::TextureFormat::Rgba8UnormSrgb,
                1,
            )))
        })
        .clone()
}

fn bench_build_geometry_with_parametrics(c: &mut Criterion) {
    let mut doc = Document::new();
    for i in 0..10 {
        doc.add_object(GeoObject::ParametricCurve3D(ParametricCurve3DObj::new(
            "cos(t)",
            "sin(t)",
            &format!("{}", (i as f64) * 0.1),
            0.0,
            std::f64::consts::TAU,
        )));
    }
    for i in 0..5 {
        doc.add_object(GeoObject::Surface3D(Surface3DObj::new(
            "x^2 + y^2",
            (-1.0 - i as f64 * 0.1, 1.0 + i as f64 * 0.1),
            (-1.0 - i as f64 * 0.1, 1.0 + i as f64 * 0.1),
        )));
    }

    let camera = Camera3D::new(800.0 / 600.0);

    if let Some(renderer) = try_headless_renderer() {
        c.bench_function("build_geometry_with_parametrics", |b| {
            b.iter(|| {
                let (vertices, indices) = renderer.build_3d_geometry(
                    black_box(&doc),
                    black_box(&camera),
                    false,
                    800.0,
                    600.0,
                    true,
                );
                black_box((vertices.len(), indices.len()));
            })
        });
    } else {
        c.bench_function("build_geometry_with_parametrics", |b| {
            b.iter(|| {
                let (vertices, indices) = Renderer::build_3d_geometry_static(
                    black_box(&doc),
                    black_box(&camera),
                    false,
                    800.0,
                    600.0,
                );
                black_box((vertices.len(), indices.len()));
            })
        });
    }
}

fn bench_build_geometry_with_vector_fields(c: &mut Criterion) {
    let mut doc = Document::new();
    for i in 0..5 {
        doc.add_object(GeoObject::VectorField2D(VectorField2DObj::new(
            "x",
            &format!("y + {}", i),
        )));
    }
    let view = view_800x600();

    c.bench_function("build_geometry_with_vector_fields", |b| {
        b.iter(|| {
            let (vertices, indices) =
                Renderer::build_geometry_static(black_box(&doc), black_box(&view), false, true);
            black_box((vertices.len(), indices.len()));
        })
    });
}

// S3 (ui-polish, 2026-10-06): escena dashed persistente del hotspot H1.
// Réplica exacta de la matemática de `stroke_runs_batched_shapes`
// (`grafito-app/src/render_2d.rs:548-580`): mismo `dash_pattern`
// (`render_2d.rs:2888-2902`), mismo loop `dist += dash + gap` con un
// `Shape::line_segment` real por pieza. Réplica y no llamada directa porque
// ambas fns son `pub(crate)` de grafito-app y este bench vive en
// grafito-render (ver journal s3-h1.md; el probe temporal ya borrado medía
// lo mismo). Escena: 40 runs x 800 pts en viewport 800x600, estilos
// dashed/dotted con w=1/2. Conteos ancla medidos en este box (x86_64):
// s3_dashed_dashed_w1=37947, s3_dashed_dotted_w1=68035,
// s3_dashed_dashed_w2=31960, s3_dashed_dotted_w2=42520 shapes.
use egui::{Pos2, Shape, Stroke};
use grafito_core::LineStyle;

const S3_RUNS: usize = 40;
const S3_PTS: usize = 800;

fn s3_dash_pattern(style: LineStyle, width: f32) -> Option<(f32, f32)> {
    let width = if width.is_finite() && width > 0.0 {
        width
    } else {
        1.0
    };
    match style {
        LineStyle::Solid => None,
        LineStyle::Dashed => Some((
            (6.0 * width).clamp(6.0, 18.0),
            (4.0 * width).clamp(3.0, 12.0),
        )),
        LineStyle::Dotted => Some((width.clamp(1.0, 3.0), (3.0 * width).clamp(2.0, 9.0))),
    }
}

fn s3_make_runs() -> Vec<Vec<Pos2>> {
    let mut runs = Vec::with_capacity(S3_RUNS);
    for r in 0..S3_RUNS {
        let phase = r as f32 * 0.37;
        let amp = 120.0 + (r as f32 % 5.0) * 40.0;
        let mut run = Vec::with_capacity(S3_PTS);
        for j in 0..S3_PTS {
            let x = j as f32 * (800.0 / S3_PTS as f32);
            let y = 300.0 + amp * (0.05 * j as f32 + phase).sin();
            run.push(Pos2::new(x, y));
        }
        runs.push(run);
    }
    runs
}

fn s3_dashed_shapes(runs: &[Vec<Pos2>], stroke: Stroke, style: LineStyle) -> Vec<Shape> {
    let mut shapes = Vec::new();
    if let Some((dash, gap)) = s3_dash_pattern(style, stroke.width) {
        for run in runs {
            let mut i = 0;
            while i + 1 < run.len() {
                let a = run[i];
                let b = run[i + 1];
                let delta = b - a;
                let len = delta.length();
                if len.is_finite() && len > 0.0 {
                    let dir = delta / len;
                    let mut dist = 0.0;
                    while dist < len {
                        let start = a + dir * dist;
                        let end = a + dir * (dist + dash).min(len);
                        shapes.push(Shape::line_segment([start, end], stroke));
                        dist += dash + gap;
                    }
                }
                i += 1;
            }
        }
    }
    shapes
}

fn bench_s3_dashed(c: &mut Criterion, name: &'static str, style: LineStyle, width: f32) {
    let runs = s3_make_runs();
    let stroke = Stroke::new(width, egui::Color32::WHITE);
    c.bench_function(name, |b| {
        b.iter(|| {
            let shapes = s3_dashed_shapes(black_box(&runs), black_box(stroke), black_box(style));
            black_box(shapes.len());
        })
    });
}

fn bench_s3_dashed_dashed_w1(c: &mut Criterion) {
    bench_s3_dashed(c, "s3_dashed_dashed_w1", LineStyle::Dashed, 1.0);
}

fn bench_s3_dashed_dotted_w1(c: &mut Criterion) {
    bench_s3_dashed(c, "s3_dashed_dotted_w1", LineStyle::Dotted, 1.0);
}

fn bench_s3_dashed_dashed_w2(c: &mut Criterion) {
    bench_s3_dashed(c, "s3_dashed_dashed_w2", LineStyle::Dashed, 2.0);
}

fn bench_s3_dashed_dotted_w2(c: &mut Criterion) {
    bench_s3_dashed(c, "s3_dashed_dotted_w2", LineStyle::Dotted, 2.0);
}

criterion_group!(
    benches,
    bench_build_geometry_many_objects,
    bench_build_geometry_with_functions,
    bench_build_geometry_with_parametrics,
    bench_build_geometry_with_vector_fields,
    bench_s3_dashed_dashed_w1,
    bench_s3_dashed_dotted_w1,
    bench_s3_dashed_dashed_w2,
    bench_s3_dashed_dotted_w2
);
criterion_main!(benches);

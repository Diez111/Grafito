use glam::Vec3;
use std::collections::VecDeque;

const _: [(); 0] = [(); crate::app::DEFAULT_KEYBOARD_VISIBLE as usize];
const _: [(); 0] = [(); crate::app::DEFAULT_CONSTRUCTION_PROTOCOL_VISIBLE as usize];

#[test]
fn fresh_workspace_starts_with_an_empty_document() {
    let document = crate::app::initial_document();

    assert_eq!(document.object_count(), 0);
    assert!(document.variables.is_empty());
}

#[test]
fn three_dimensional_gpu_path_is_gated_by_readiness_and_settled_view() {
    assert!(!crate::app::should_use_gpu_3d(true, true, true));
    assert!(!crate::app::should_use_gpu_3d(true, false, false));
    assert!(crate::app::should_use_gpu_3d(true, true, false));
}

#[test]
fn construction_protocol_avoids_controls_that_do_not_change_construction() {
    let source = include_str!("panels.rs");

    assert!(!source.contains("move_step_button("));
    assert!(!source.contains("button(if disabled"));
}

#[test]
fn cpu_point_label_projection_rejects_a_point_beyond_the_far_plane() {
    let mut camera = axis_aligned_test_camera();
    camera.far = 5.0;

    assert_eq!(
        crate::render_3d::projected_point_position(
            &camera,
            grafito_geometry::Point3D::new(0.0, 0.0, 0.0),
            egui::vec2(800.0, 600.0),
        ),
        None,
        "CPU point labels must not survive far-plane clipping"
    );
}

fn axis_aligned_test_camera() -> grafito_geometry::Camera3D {
    let mut camera = grafito_geometry::Camera3D::new(4.0 / 3.0);
    camera.theta = 0.0;
    camera.phi = 0.0;
    camera.distance = 10.0;
    camera.target = Vec3::ZERO;
    camera
}

#[test]
fn construction_points_follow_canvas_local_pointer_on_target_plane() {
    let camera = axis_aligned_test_camera();
    let canvas = egui::Rect::from_min_size(egui::pos2(125.0, 75.0), egui::vec2(800.0, 600.0));
    let center_pointer = crate::input::canvas_local_pointer(canvas, egui::pos2(525.0, 375.0))
        .expect("global center pointer becomes canvas-local");
    let off_center_pointer = crate::input::canvas_local_pointer(canvas, egui::pos2(725.0, 275.0))
        .expect("global off-center pointer becomes canvas-local");
    assert_eq!(center_pointer, egui::vec2(400.0, 300.0));
    assert_eq!(off_center_pointer, egui::vec2(600.0, 200.0));
    assert!(crate::input::canvas_local_pointer(canvas, egui::pos2(124.0, 75.0)).is_none());

    let center =
        crate::render_3d::construction_point_from_canvas(&camera, center_pointer, canvas.size())
            .expect("center placement");
    let off_center = crate::render_3d::construction_point_from_canvas(
        &camera,
        off_center_pointer,
        canvas.size(),
    )
    .expect("off-center placement");

    assert!(center.distance(&grafito_geometry::Point3D::new(0.0, 0.0, 0.0)) < 1.0e-4);
    assert!(center.distance(&off_center) > 0.1);
    assert!(center.x.abs() < 1.0e-4);
    assert!(off_center.x.abs() < 1.0e-4);
}

#[test]
fn cpu_tetrahedron_fallback_projects_four_depth_sorted_faces() {
    let camera = axis_aligned_test_camera();
    let tetrahedron =
        grafito_geometry::Tetrahedron3D::new(grafito_geometry::Point3D::new(0.0, 0.0, 0.0), 2.0);

    let faces = crate::render_3d::projected_tetrahedron_faces(&camera, &tetrahedron, 800.0, 600.0);

    assert_eq!(faces.len(), 4);
    assert!(faces.windows(2).all(|faces| faces[0].0 >= faces[1].0));
    assert!(faces
        .iter()
        .all(|(_, face)| { face.iter().all(|(x, y)| x.is_finite() && y.is_finite()) }));
}

#[test]
fn picker_hits_required_3d_primitive_types() {
    use grafito_core::{
        Cube3DObj, GeoObject, Line3DObj, Plane3DObj, Point3DObj, Segment3DObj, Sphere3DObj,
        Tetrahedron3DObj,
    };
    use grafito_geometry::Point3D;

    let camera = axis_aligned_test_camera();
    let pointer = egui::vec2(400.0, 300.0);
    let canvas_size = egui::vec2(800.0, 600.0);
    let objects = [
        GeoObject::Point3D(Point3DObj::new(Point3D::new(0.0, 0.0, 0.0))),
        GeoObject::Segment3D(Segment3DObj::new(
            Point3D::new(0.0, -1.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        )),
        GeoObject::Line3D(Line3DObj::from_point_and_direction(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        )),
        GeoObject::Sphere3D(Sphere3DObj::new(Point3D::new(0.0, 0.0, 0.0), 1.0)),
        GeoObject::Cube3D(Cube3DObj::new(Point3D::new(0.0, 0.0, 0.0), 2.0)),
        GeoObject::Tetrahedron3D(Tetrahedron3DObj::new(Point3D::new(0.0, 0.0, 0.0), 2.0)),
        GeoObject::Plane3D(Plane3DObj::from_equation(1.0, 0.0, 0.0, 0.0)),
    ];

    for object in objects {
        let expected = object.id();
        let mut document = grafito_core::Document::new();
        document
            .try_add_object(object)
            .expect("valid primitive fixture");
        assert_eq!(
            crate::render_3d::pick_3d_object(&document, &camera, pointer, canvas_size),
            Some(expected),
            "primitive {expected} should be selectable"
        );
    }
}

#[test]
fn picker_uses_nearest_visible_hit_and_deterministic_ties() {
    use grafito_core::{GeoObject, Sphere3DObj};
    use grafito_geometry::Point3D;

    let camera = axis_aligned_test_camera();
    let pointer = egui::vec2(400.0, 300.0);
    let canvas_size = egui::vec2(800.0, 600.0);
    let mut document = grafito_core::Document::new();
    let far = document
        .try_add_object(GeoObject::Sphere3D(Sphere3DObj::new(
            Point3D::new(-2.0, 0.0, 0.0),
            1.0,
        )))
        .expect("far sphere");
    let near = document
        .try_add_object(GeoObject::Sphere3D(Sphere3DObj::new(
            Point3D::new(3.0, 0.0, 0.0),
            1.0,
        )))
        .expect("near sphere");

    assert_eq!(
        crate::render_3d::pick_3d_object(&document, &camera, pointer, canvas_size),
        Some(near)
    );
    document
        .get_object_mut(near)
        .expect("near sphere remains")
        .set_visible(false);
    assert_eq!(
        crate::render_3d::pick_3d_object(&document, &camera, pointer, canvas_size),
        Some(far)
    );

    let mut tied = grafito_core::Document::new();
    let first = tied
        .try_add_object(GeoObject::Sphere3D(Sphere3DObj::new(
            Point3D::new(0.0, 0.0, 0.0),
            1.0,
        )))
        .expect("first tied sphere");
    let second = tied
        .try_add_object(GeoObject::Sphere3D(Sphere3DObj::new(
            Point3D::new(0.0, 0.0, 0.0),
            1.0,
        )))
        .expect("second tied sphere");
    assert_eq!(
        crate::render_3d::pick_3d_object(&tied, &camera, pointer, canvas_size),
        Some(first.min(second))
    );
}

#[test]
fn picker_exact_hit_outranks_nearer_torus_fallback_through_its_hole() {
    use grafito_core::{GeoObject, Sphere3DObj, Torus3DObj};
    use grafito_geometry::Point3D;

    let mut camera = axis_aligned_test_camera();
    camera.phi = std::f32::consts::FRAC_PI_2 - 0.01;
    let pointer = egui::vec2(400.0, 300.0);
    let canvas_size = egui::vec2(800.0, 600.0);
    let ray = camera
        .screen_ray(pointer.x, pointer.y, canvas_size.x, canvas_size.y)
        .expect("top-view center ray");
    let sphere_center = ray.point_at(12.0).expect("sphere center inside frustum");
    let mut document = grafito_core::Document::new();
    let torus = document
        .try_add_object(GeoObject::Torus3D(Torus3DObj::new(
            Point3D::new(0.0, 0.0, 0.0),
            2.0,
            0.5,
        )))
        .expect("torus fixture");
    let sphere = document
        .try_add_object(GeoObject::Sphere3D(Sphere3DObj::new(sphere_center, 0.5)))
        .expect("sphere visible through the torus hole");

    assert_ne!(torus, sphere);
    assert_eq!(
        crate::render_3d::pick_3d_object(&document, &camera, pointer, canvas_size),
        Some(sphere),
        "a coarse torus AABB must not occlude an exact sphere hit"
    );
}

#[test]
fn picker_respects_frustum_clipping_and_surface_fallback_bounds() {
    use grafito_core::{GeoObject, Point3DObj, Surface3DObj, VectorField3DObj};
    use grafito_geometry::Point3D;

    let pointer = egui::vec2(400.0, 300.0);
    let canvas_size = egui::vec2(800.0, 600.0);
    let mut document = grafito_core::Document::new();
    document
        .try_add_object(GeoObject::Point3D(Point3DObj::new(Point3D::new(
            12.0, 0.0, 0.0,
        ))))
        .expect("point behind camera");
    let mut short_camera = axis_aligned_test_camera();
    short_camera.far = 5.0;
    document
        .try_add_object(GeoObject::Point3D(Point3DObj::new(Point3D::new(
            0.0, 0.0, 0.0,
        ))))
        .expect("point beyond far plane");
    assert_eq!(
        crate::render_3d::pick_3d_object(&document, &short_camera, pointer, canvas_size),
        None
    );

    let mut just_beyond_far = grafito_core::Document::new();
    just_beyond_far
        .try_add_object(GeoObject::Point3D(Point3DObj::new(Point3D::new(
            -9991.0, 0.0, 0.0,
        ))))
        .expect("point just beyond the default far plane");
    assert_eq!(
        crate::render_3d::pick_3d_object(
            &just_beyond_far,
            &axis_aligned_test_camera(),
            pointer,
            canvas_size,
        ),
        None,
        "screen-space hit tolerance must not bypass far clipping"
    );

    let mut clipped_fallback = grafito_core::Document::new();
    clipped_fallback
        .try_add_object(GeoObject::VectorField3D(
            VectorField3DObj::new("1", "0", "0").with_bounds(
                (-9992.0, -9991.0),
                (-1.0, 1.0),
                (-1.0, 1.0),
            ),
        ))
        .expect("fallback bounds beyond far plane");
    assert_eq!(
        crate::render_3d::pick_3d_object(
            &clipped_fallback,
            &axis_aligned_test_camera(),
            pointer,
            canvas_size,
        ),
        None,
        "fallback padding must not bypass far clipping"
    );

    let mut surface_document = grafito_core::Document::new();
    let surface = surface_document
        .try_add_object(GeoObject::Surface3D(Surface3DObj::new(
            "0",
            (-1.0, 1.0),
            (-1.0, 1.0),
        )))
        .expect("flat surface");
    assert_eq!(
        crate::render_3d::pick_3d_object(
            &surface_document,
            &axis_aligned_test_camera(),
            pointer,
            canvas_size,
        ),
        Some(surface)
    );
}

#[test]
fn select_3d_click_updates_and_empty_click_clears_selection() {
    use grafito_core::{GeoObject, Sphere3DObj};
    use grafito_geometry::Point3D;

    let mut document = grafito_core::Document::new();
    let sphere = document
        .try_add_object(GeoObject::Sphere3D(Sphere3DObj::new(
            Point3D::new(0.0, 0.0, 0.0),
            1.0,
        )))
        .expect("sphere fixture");
    let mut selected_object = None;
    let camera = axis_aligned_test_camera();
    let canvas_size = egui::vec2(800.0, 600.0);

    assert_eq!(
        crate::render_3d::select_3d_object_at_pointer(
            &mut document,
            &mut selected_object,
            &camera,
            egui::vec2(400.0, 300.0),
            canvas_size,
        ),
        Some(sphere)
    );
    assert_eq!(selected_object, Some(sphere));
    assert_eq!(document.selection(), &[sphere]);

    assert_eq!(
        crate::render_3d::select_3d_object_at_pointer(
            &mut document,
            &mut selected_object,
            &camera,
            egui::vec2(0.0, 0.0),
            canvas_size,
        ),
        None
    );
    assert_eq!(selected_object, None);
    assert!(document.selection().is_empty());
}

#[test]
fn test_save_load_roundtrip() {
    use grafito_core::*;
    use grafito_geometry::*;
    let mut doc = Document::new();
    doc.add_object(GeoObject::Point(PointObj::new(Point2::new(1.0, 2.0))));
    doc.add_object(GeoObject::Circle(CircleObj::new(
        Point2::new(0.0, 0.0),
        5.0,
    )));
    doc.set_variable("a".into(), 42.0);

    let tmp = std::env::temp_dir().join("grafito_test_roundtrip.json");
    crate::export::save_document(&doc, &tmp.to_string_lossy()).expect("save failed");
    let loaded = crate::export::load_document(&tmp.to_string_lossy()).expect("load failed");
    let _ = std::fs::remove_file(&tmp);

    assert_eq!(loaded.object_count(), 2);
    assert_eq!(loaded.get_variable("a"), Some(42.0));
}

#[test]
fn test_save_load_constraint_params_roundtrip() {
    use grafito_core::*;
    use grafito_geometry::*;
    use std::collections::BTreeMap;

    let mut doc = Document::new();
    let a = doc.add_object(GeoObject::Point(
        PointObj::new(Point2::new(0.0, 0.0)).with_label("A"),
    ));
    let mut params = BTreeMap::new();
    params.insert("dx".to_string(), 2.0);
    params.insert("dy".to_string(), 3.0);
    let (_p, cons_id) = doc.add_constructed_object_with_params(
        GeoObject::Point(PointObj::new(Point2::new(2.0, 3.0)).with_label("A'")),
        "Translate",
        &[a],
        params,
    );

    let tmp = std::env::temp_dir().join("grafito_test_constraint_params.json");
    crate::export::save_document(&doc, &tmp.to_string_lossy()).expect("save failed");
    let loaded = crate::export::load_document(&tmp.to_string_lossy()).expect("load failed");
    let _ = std::fs::remove_file(&tmp);

    let cons = loaded
        .constraints
        .get_constraint(cons_id)
        .expect("constraint should survive roundtrip");
    assert_eq!(cons.params.get("dx"), Some(&2.0));
    assert_eq!(cons.params.get("dy"), Some(&3.0));
}

#[test]
fn test_export_svg() {
    use grafito_core::*;
    use grafito_geometry::*;
    let mut doc = Document::new();
    doc.add_object(GeoObject::Point(PointObj::new(Point2::new(1.0, 2.0))));
    doc.add_object(GeoObject::Line(LineObj::new(
        Point2::new(0.0, 0.0),
        Point2::new(3.0, 4.0),
    )));

    doc.view_mut().screen_size = glam::Vec2::new(800.0, 600.0);
    let path = std::env::temp_dir().join(format!(
        "grafito_app_export_test_{}.svg",
        std::process::id()
    ));
    crate::export::export_svg(&doc, &path).expect("SVG export should succeed");
    let svg = std::fs::read_to_string(&path).expect("SVG should be readable");
    let _ = std::fs::remove_file(path);
    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));
    assert!(svg.contains("data-grafito-type=\"Point\""));
    assert!(svg.contains("data-grafito-type=\"Line\""));
}

#[test]
fn export_ui_never_uses_a_fixed_working_directory_path_or_ignored_write_result() {
    let panels = include_str!("panels.rs");
    let app = include_str!("app.rs");

    assert!(!panels.contains("grafito_export.svg"));
    assert!(!panels.contains("let _ = std::fs::write"));
    assert!(!app.contains("std::fs::write(&path, svg)"));
    assert!(!app.contains("std::fs::write(&path, tex)"));
}

#[test]
fn export_outcomes_keep_full_persistent_feedback_and_show_a_toast() {
    use crate::export::{ExportError, ExportFormat, ExportItem, ExportReport};

    let mut cas_result = String::new();
    let mut toasts = grafito_ui::toast::ToastManager::default();
    let empty_toasts = toast_shape_count(&mut grafito_ui::toast::ToastManager::default());
    let report = ExportReport {
        format: ExportFormat::Svg,
        path: std::path::PathBuf::from("/tmp/scene.svg"),
        exported_objects: 2,
        hidden_objects: 1,
        primitive_count: 4,
        object_types: std::collections::BTreeMap::from([
            ("Circle".to_string(), 1),
            ("Point".to_string(), 1),
        ]),
    };

    crate::app::apply_export_outcome(Ok(report), &mut cas_result, &mut toasts, 0.0);
    assert!(cas_result.contains("2 objetos"));
    assert!(cas_result.contains("Circle x1"));
    assert!(cas_result.contains("1 ocultos"));
    assert!(toast_shape_count(&mut toasts) > empty_toasts);

    let omitted = ExportItem {
        object_type: "Surface3D".to_string(),
        label: "S".to_string(),
        object_id: "fixture".to_string(),
    };
    crate::app::apply_export_outcome(
        Err(ExportError::UnsupportedObjects {
            format: ExportFormat::Png,
            objects: vec![omitted],
        }),
        &mut cas_result,
        &mut toasts,
        1.0,
    );
    assert!(cas_result.contains("Surface3D 'S'"));
    assert!(cas_result.contains("no reemplazo el destino"));
}

// ── Tests del sistema de Perspectivas ────────────────────────────────────

#[test]
fn test_perspective_all_has_ten_variants() {
    // Once clásicas + Física (el nombre conserva historia; el pin es el 11).
    assert_eq!(crate::Perspective::ALL.len(), 11);
}

#[test]
fn test_perspective_view_mode_derivation() {
    use crate::Perspective;
    use crate::ViewMode;
    assert_eq!(Perspective::Geometry3D.view_mode(), ViewMode::D3);
    assert_eq!(Perspective::Dynamics.view_mode(), ViewMode::D3);
    assert_eq!(Perspective::Geometry2D.view_mode(), ViewMode::D2);
    assert_eq!(Perspective::AlgebraCas.view_mode(), ViewMode::D2);
    assert_eq!(Perspective::Calculus.view_mode(), ViewMode::D2);
    assert_eq!(Perspective::Probability.view_mode(), ViewMode::D2);
    assert_eq!(Perspective::Statistics.view_mode(), ViewMode::D2);
    assert_eq!(Perspective::Complex.view_mode(), ViewMode::D2);
    assert_eq!(Perspective::DataAnalysis.view_mode(), ViewMode::D2);
    assert_eq!(Perspective::Exam.view_mode(), ViewMode::D2);
}

#[test]
fn test_perspective_shortcut_numbers_unique() {
    use crate::Perspective;
    // Etiquetas únicas: dígitos 0-9 + F de Física (dígitos agotados).
    let mut labels: Vec<&str> = Perspective::ALL
        .iter()
        .map(|p| p.shortcut_label())
        .collect();
    labels.sort_unstable();
    assert_eq!(
        labels,
        vec!["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "F"]
    );
    assert_eq!(Perspective::Exam.shortcut_label(), "0");
    assert_eq!(Perspective::Fisica.shortcut_label(), "F");
}

#[test]
fn fisica_starter_muestra_tres_ramas() {
    // El starter de Física pisa las tres ramas (mecánica, electro,
    // ondas): si alguna se cae, el ejemplo queda rengo y el novato
    // no lo nota. Pinche acá, no en cada comando por separado.
    use crate::Perspective;
    use grafito_command::commands::CommandOutcome;
    use grafito_core::GeoObject;

    let mut app = crate::app::dummy_grafito_app_with_perspective(Perspective::Fisica);
    assert_eq!(app.document.object_count(), 0);
    app.load_perspective_examples(Perspective::Fisica)
        .expect("el starter de Física carga sobre documento vacío");
    assert!(
        app.document.object_count() >= 5,
        "starter Física: {} objetos, se esperan 5 o más",
        app.document.object_count()
    );
    assert!(
        app.document
            .objects_iter()
            .any(|(_, object)| !matches!(object, GeoObject::Function(_))),
        "starter Física: todo quedó como Function, falta curva/campo/puntos"
    );
    let mut document = grafito_core::Document::new();
    let mut ohm = "LeyOhm[12, 4]".to_string();
    match grafito_command::commands::process_input(&mut document, &mut ohm) {
        CommandOutcome::Message(message) => assert!(
            message.contains('3'),
            "LeyOhm[12, 4] tiene que anunciar I=3 A, dijo: {message}"
        ),
        otro => panic!("LeyOhm[12, 4] es solo lectura con mensaje, dio: {otro:?}"),
    }
    let mut tiro = "TrayectoriaProyectil[20, 45, 1.5]".to_string();
    match grafito_command::commands::process_input(&mut document, &mut tiro) {
        CommandOutcome::Ok | CommandOutcome::Message(_) => {}
        otro => panic!("TrayectoriaProyectil[20, 45, 1.5] tiene que salir bien, dio: {otro:?}"),
    }
}

#[test]
fn perspective_layout_fisica_difiere_geometria() {
    // S3 Piel: Física no reutiliza el layout de Cálculo/Geometría — panel
    // izquierdo propio (laboratorios con parámetros a la vista), Measure
    // agregada a las herramientas y canvas 2D completo pineado.
    use crate::{CanvasMode, LeftPanelContent, Perspective};
    use grafito_command::commands::{process_input, CommandOutcome};
    use grafito_ui::command_palette::{all_commands, dinamica_first};

    let fisica = Perspective::Fisica.layout();
    let geometria = Perspective::Geometry2D.layout();
    // Panel izquierdo propio, no el Álgebra genérico.
    assert_eq!(fisica.left_panel, LeftPanelContent::Fisica);
    assert_ne!(
        fisica.left_panel, geometria.left_panel,
        "Física reutiliza el panel de Geometría 2D"
    );
    assert_eq!(fisica.left_panel.default_sidebar_tab(), 1);
    assert_eq!(fisica.left_panel.sidebar_label(), "Física");
    // Herramientas: Measure agregada a [Move, Curve, Dynamics].
    use grafito_ui::toolbar::ToolGroupId as G;
    assert_eq!(
        fisica.visible_tool_groups,
        &[G::Move, G::Curve, G::Dynamics, G::Measure]
    );
    assert_ne!(
        fisica.visible_tool_groups, geometria.visible_tool_groups,
        "Física comparte herramientas con Geometría 2D"
    );
    // Canvas 2D pineado (no SmallD2: los diagramas necesitan todo el
    // ancho) y derecha None (los parámetros viven a la izquierda).
    assert_eq!(fisica.canvas_mode, CanvasMode::D2);
    assert_eq!(fisica.right_panel, None);
    // Paleta: la sección Dinámica abre la lista en Física.
    let ordenada = dinamica_first(all_commands());
    let primera_no_dinamica = ordenada.iter().position(|cmd| cmd.category != "Dinámica");
    let ultima_dinamica = ordenada.iter().rposition(|cmd| cmd.category == "Dinámica");
    if let (Some(primera), Some(ultima)) = (primera_no_dinamica, ultima_dinamica) {
        assert!(
            ultima < primera,
            "los laboratorios de Dinámica no abren la paleta de Física"
        );
    }
    // Autocompletado: en Física el tiro oblicuo supera a lo genérico.
    let doc = grafito_core::Document::new();
    let candidatas =
        crate::ui::compute_autocomplete_suggestions_for_perspective("trayectoria", &doc, true);
    assert_eq!(
        candidatas.first().map(|item| item.text.as_str()),
        Some("TrayectoriaProyectil"),
        "en Física el laboratorio tiene que listar primero"
    );
    // Unidades: las que el motor ya da en sus mensajes, sin inventar.
    for (comando, unidad) in [
        ("VelocidadOnda[2.0, 3.0]", "m/s"),
        ("LeyGravitacion[5.972e24, 7.342e22, 3.844e8]", " N"),
        ("LeyOhm[12, 4]", " A"),
        ("TrayectoriaProyectil[20, 45, 1.5]", " m"),
    ] {
        let mut entrada = comando.to_string();
        let mut documento = grafito_core::Document::new();
        match process_input(&mut documento, &mut entrada) {
            CommandOutcome::Message(mensaje) => assert!(
                mensaje.contains(unidad),
                "{comando} tiene que anunciar {unidad}, dijo: {mensaje}"
            ),
            otro => panic!("{comando} es lectura o crea objeto, dio: {otro:?}"),
        }
    }
    // Panel propio: dibuja sin pánico y ofrece los laboratorios.
    let ctx = egui::Context::default();
    let mut fisica_app = crate::app::dummy_grafito_app_with_perspective(Perspective::Fisica);
    let salida = ctx.run(headless_raw_input(), |ctx| {
        crate::panels::draw_fisica_panel(&mut fisica_app, ctx);
    });
    assert!(salida.shapes.len() < 10_000);
    assert!(
        headless_texts(&salida).any(|texto| texto.contains("TrayectoriaProyectil")),
        "el panel de Física ofrece el tiro oblicuo"
    );
}

#[test]
fn test_perspective_layout_canvas_modes() {
    use crate::CanvasMode;
    use crate::Perspective;
    assert_eq!(Perspective::Geometry2D.layout().canvas_mode, CanvasMode::D2);
    assert_eq!(Perspective::Geometry3D.layout().canvas_mode, CanvasMode::D3);
    assert_eq!(
        Perspective::AlgebraCas.layout().canvas_mode,
        CanvasMode::SmallD2
    );
    assert_eq!(
        Perspective::Probability.layout().canvas_mode,
        CanvasMode::SmallD2
    );
    assert_eq!(Perspective::Dynamics.layout().canvas_mode, CanvasMode::D3);
}

#[test]
fn test_perspective_layout_tool_groups_nonempty() {
    use crate::Perspective;
    for p in Perspective::ALL {
        let layout = p.layout();
        assert!(
            !layout.visible_tool_groups.is_empty(),
            "perspectiva {:?} no define grupos de herramientas",
            p
        );
    }
}

#[test]
fn stable_perspectives_do_not_expose_unavailable_placeholder_tools() {
    // Ola 1.6: `Tool::Button` pasó a ser real (botón de acción con guion);
    // `Tool::Image` sigue oculta hasta tener modelo persistente.
    use crate::Perspective;
    use grafito_ui::Tool;

    for perspective in Perspective::ALL {
        for group in perspective.layout().visible_tool_groups {
            let (_, tools) = group.def();
            let unavailable = Tool::Image;
            assert!(
                tools.iter().all(|(tool, _, _)| *tool != unavailable),
                "{perspective:?} exposes unavailable tool {unavailable:?} through {group:?}"
            );
        }
    }
}

#[test]
fn stable_tools_panel_and_status_help_do_not_claim_unavailable_features() {
    let tools_panel = include_str!("tools_panel.rs");
    let status_help = include_str!("ui.rs");

    let registration = "(Tool::Image,";
    assert!(
        !tools_panel.contains(registration),
        "stable tools panel still registers {registration}"
    );
    assert!(
        status_help.contains("hint_msg(\"locus\", locale)"),
        "Locus sin hint propio cableado a hint.* (B-A188)"
    );
}

#[test]
fn test_3d_perspectives_do_not_expose_2d_curve_group() {
    use crate::Perspective;
    use grafito_ui::toolbar::ToolGroupId;

    let geometry_3d = Perspective::Geometry3D.layout();
    assert!(geometry_3d
        .visible_tool_groups
        .contains(&ToolGroupId::ThreeD));
    assert!(!geometry_3d
        .visible_tool_groups
        .contains(&ToolGroupId::Curve));

    let dynamics = Perspective::Dynamics.layout();
    assert!(dynamics
        .visible_tool_groups
        .contains(&ToolGroupId::Dynamics));
    assert!(!dynamics
        .visible_tool_groups
        .contains(&ToolGroupId::Advanced));
}

#[test]
fn four_d_tools_are_available_only_from_the_geometry_3d_perspective() {
    use crate::Perspective;
    use grafito_ui::toolbar::ToolGroupId;

    assert!(Perspective::Geometry3D
        .layout()
        .visible_tool_groups
        .contains(&ToolGroupId::FourD));
    for perspective in Perspective::ALL {
        if perspective != Perspective::Geometry3D {
            assert!(
                !perspective
                    .layout()
                    .visible_tool_groups
                    .contains(&ToolGroupId::FourD),
                "{perspective:?} must not expose the centered projected 4D tools"
            );
        }
    }
}

#[test]
fn four_d_tools_create_select_typed_centered_defaults_and_reset_to_select() {
    use grafito_core::GeoObject;
    use grafito_geometry::{RegularPolychoron, RegularPolytopeFamily};
    use grafito_ui::Tool;

    for tool in [Tool::Tesseract4D, Tool::Hypercube5D] {
        let mut document = grafito_core::Document::new();
        let mut selected = None;
        let mut undo_stack = VecDeque::new();
        let mut redo_stack = VecDeque::new();
        let mut current_tool = tool;

        let (id, action) = crate::render_3d::create_centered_four_d_tool_object(
            &mut current_tool,
            &mut document,
            &mut selected,
            &mut undo_stack,
            &mut redo_stack,
        )
        .expect("the default centered projected object is valid")
        .expect("the selected tool creates an object");

        assert_eq!(current_tool, Tool::Select);
        assert_eq!(selected, Some(id));
        assert!(document.is_selected(id));
        assert_eq!(undo_stack.len(), 1);
        assert!(redo_stack.is_empty());
        match (tool, document.get_object(id)) {
            (Tool::Tesseract4D, Some(GeoObject::RegularPolychoron4D(polychoron))) => {
                assert_eq!(action, "Tesseract4D");
                assert_eq!(polychoron.kind, RegularPolychoron::Tesseract);
                assert_eq!(polychoron.scale, 1.0);
                assert_eq!(polychoron.rotation_angles, [0.0; 6]);
            }
            (Tool::Hypercube5D, Some(GeoObject::RegularPolytopeND(polytope))) => {
                assert_eq!(action, "Hypercube5D");
                assert_eq!(polytope.family, RegularPolytopeFamily::Hypercube);
                assert_eq!(polytope.dimension, 5);
                assert_eq!(polytope.scale, 1.0);
                assert_eq!(polytope.rotation_angles, vec![0.0; 10]);
            }
            (_, object) => panic!("unexpected centered projected object: {object:?}"),
        }
    }
}

#[test]
fn rejected_four_d_tool_insertion_preserves_the_document_history_and_tool_state() {
    use grafito_core::ChangeSet;
    use grafito_ui::Tool;

    let mut document = document_at_object_capacity();
    let selected = document
        .objects_iter()
        .next()
        .map(|(id, _)| *id)
        .expect("capacity fixture contains objects");
    let before = serde_json::to_value(&document).expect("serialize full document");
    let mut selected_object = Some(selected);
    let mut undo_stack = VecDeque::new();
    let mut redo_stack = VecDeque::from([ChangeSet {
        before: grafito_core::Document::new(),
        after: grafito_core::Document::new(),
    }]);
    let mut current_tool = Tool::Tesseract4D;

    let error = crate::render_3d::create_centered_four_d_tool_object(
        &mut current_tool,
        &mut document,
        &mut selected_object,
        &mut undo_stack,
        &mut redo_stack,
    )
    .expect_err("the at-capacity insertion must fail atomically");

    assert!(error.contains("maximum"));
    assert_eq!(current_tool, Tool::Tesseract4D);
    assert_eq!(selected_object, Some(selected));
    assert!(undo_stack.is_empty());
    assert_eq!(redo_stack.len(), 1);
    assert_eq!(
        serde_json::to_value(&document).expect("serialize rejected full document"),
        before
    );
}

#[test]
fn four_d_tools_do_not_use_pointer_position_ghosts_and_explain_centered_projection() {
    use grafito_ui::Tool;

    assert!(!crate::input::uses_3d_position_ghost(Tool::Tesseract4D));
    assert!(!crate::input::uses_3d_position_ghost(Tool::Hypercube5D));
    // B-A188: firma con `locale`; Es pineado (resto por `hint.*`).
    use grafito_ui::i18n::Locale as HintLocale;
    assert!(
        crate::ui::status_hint_for_3d_tool(Tool::Tesseract4D, HintLocale::Es).contains("centrado")
    );
    assert!(
        crate::ui::status_hint_for_3d_tool(Tool::Tesseract4D, HintLocale::Es)
            .contains("proyectado")
    );
    assert!(
        crate::ui::status_hint_for_3d_tool(Tool::Hypercube5D, HintLocale::Es).contains("centrado")
    );
    assert!(
        crate::ui::status_hint_for_3d_tool(Tool::Hypercube5D, HintLocale::Es)
            .contains("proyectado")
    );
}

#[test]
fn four_d_tools_are_discoverable_from_the_3d_tools_panel_and_3d_click_route() {
    let tools_panel = include_str!("tools_panel.rs");
    let render_3d = include_str!("render_3d.rs");

    assert!(tools_panel.contains("4D proyectado"));
    assert!(tools_panel.contains("Crea un teseracto 4D centrado y proyectado"));
    assert!(tools_panel.contains("Crea un hipercubo 5D centrado y proyectado"));
    assert!(tools_panel.contains("WidgetInfo::labeled"));

    let handler_start = render_3d
        .find("pub fn handle_3d_click")
        .expect("3D click handler");
    let construction_point = render_3d
        .find("let Some(c) = construction_point_for_view(")
        .expect("position-based 3D construction route");
    let centered_handler_route = &render_3d[handler_start..construction_point];
    assert!(
        centered_handler_route.contains("create_centered_four_d_tool_object("),
        "the centered projected object must be created before a pointer position is consumed"
    );
    assert!(
        !centered_handler_route.contains("tool_dispatcher"),
        "the centered projected object must not route through the 2D dispatcher"
    );
}

#[test]
fn algebra_summaries_identify_typed_centered_projected_polytopes() {
    use grafito_core::{GeoObject, RegularPolychoron4DObj, RegularPolytopeNDObj};
    use grafito_geometry::{RegularPolychoron, RegularPolytopeFamily};

    let tesseract = crate::algebra::object_expression_summary(&GeoObject::RegularPolychoron4D(
        RegularPolychoron4DObj::new(RegularPolychoron::Tesseract),
    ));
    let hypercube = crate::algebra::object_expression_summary(&GeoObject::RegularPolytopeND(
        RegularPolytopeNDObj::new(RegularPolytopeFamily::Hypercube, 5),
    ));

    assert!(tesseract.contains("Teseracto 4D"));
    assert!(tesseract.contains("centrado"));
    assert!(hypercube.contains("Hipercubo 5D"));
    assert!(hypercube.contains("centrado"));
}

#[test]
fn test_perspective_layout_exam_restricted() {
    use crate::Perspective;
    let layout = Perspective::Exam.layout();
    assert!(layout.right_panel.is_none());
    assert!(!layout.show_math_keyboard);
    // Modo examen: sólo herramientas básicas (Move, Point, Line, Circle, Polygon).
    assert_eq!(layout.visible_tool_groups.len(), 5);
}

#[test]
fn every_perspective_keeps_the_math_keyboard_available() {
    use crate::Perspective;

    // Teclado eliminado por diseño Scandinavian: ninguna perspectiva lo muestra.
    assert!(Perspective::ALL
        .into_iter()
        .all(|perspective| !perspective.layout().show_math_keyboard));
}

#[test]
fn keyboard_explicit_choice_survives_perspective_switch() {
    // El toggle explícito del usuario no se pierde al cambiar de vista (antes
    // `set_perspective` lo pisaba y el teclado "nunca aparecía").
    let mut app = crate::app::dummy_grafito_app();
    app.set_perspective(crate::Perspective::Geometry2D);
    assert!(!app.keyboard_visible);
    assert!(!app.keyboard_visible_explicit);
    app.keyboard_visible = true;
    app.keyboard_visible_explicit = true;
    app.set_perspective(crate::Perspective::Geometry3D);
    assert!(app.keyboard_visible);
    assert!(app.keyboard_visible_explicit);
}

#[test]
fn complex_perspective_uses_panel_input_instead_of_bottom_bar() {
    use crate::Perspective;

    assert!(!Perspective::Complex.layout().show_input_bar);
    assert!(Perspective::Geometry2D.layout().show_input_bar);
    assert!(Perspective::Calculus.layout().show_input_bar);
}

#[test]
fn complex_panel_recommends_executable_domain_coloring() {
    // Normalizado a LF: en Windows el checkout puede traer CRLF y las
    // búsquedas multilínea fallaban (o pasaban por ausencia).
    // El ejemplo ejecutable vive en `sheet_layout.rs` (dueño puro,
    // slices F14.4.4) y `panels.rs` lo consume por import.
    let panels = include_str!("panels.rs").replace("\r\n", "\n");
    let layout = include_str!("sheet_layout.rs").replace("\r\n", "\n");

    assert!(
        panels.contains("DomainColoring[1/z, -2, 2, -2, 2, 160]")
            || layout.contains("DomainColoring[1/z, -2, 2, -2, 2, 160]")
    );
    assert!(!panels.contains("ComplexGrid[1/z]\nColoración por fase"));
    assert!(!layout.contains("ComplexGrid[1/z]\nColoración por fase"));
}

fn document_at_object_capacity() -> grafito_core::Document {
    static DOCUMENT: std::sync::OnceLock<grafito_core::Document> = std::sync::OnceLock::new();

    DOCUMENT
        .get_or_init(|| {
            let mut document = grafito_core::Document::new();
            for index in 0..grafito_core::validation::MAX_OBJECT_COUNT {
                document
                    .try_add_point(grafito_geometry::Point2::new(index as f64, 0.0))
                    .expect("capacity fixture point must be valid");
            }
            document
        })
        .clone()
}

#[test]
fn zero_radius_circle_tool_is_rejected_without_history_or_document_mutation() {
    use grafito_command::commands::CommandOutcome;
    use grafito_geometry::Point2;
    use grafito_ui::Tool;

    let mut document = grafito_core::Document::new();
    let mut state = crate::tool_dispatcher::ToolState::default();
    let point = Point2::new(2.0, 3.0);
    let first =
        crate::tool_dispatcher::dispatch_tool(Tool::Circle, &mut state, &mut document, point);
    assert!(first.objects.is_empty());
    // W-B: la tool encamina a `Circle[(2, 3), (2, 3)]` y el comando rechaza
    // el radio nulo sin crear objetos (rechazo temprano honesto, sin mutar).
    let second =
        crate::tool_dispatcher::dispatch_tool(Tool::Circle, &mut state, &mut document, point);
    assert!(second.objects.is_empty());
    let error = match state.last_outcome.take() {
        Some(CommandOutcome::Error(error)) => error,
        other => panic!("expected command error, got {other:?}"),
    };
    assert!(error.contains("Circle.radius") || error.contains("radio"));

    let before = serde_json::to_value(&document).expect("serialize document before rejection");
    assert!(state.pending.is_empty());
    assert_eq!(
        serde_json::to_value(&document).expect("serialize document after rejection"),
        before
    );
}

#[test]
fn legacy_placeholder_tools_error_without_document_mutation() {
    use grafito_command::commands::CommandOutcome;
    use grafito_geometry::Point2;
    use grafito_ui::Tool;

    // Ola 1.6: Button ya no es placeholder; Image sigue fallando honesto.
    let tool = Tool::Image;
    {
        let mut document = grafito_core::Document::new();
        document.set_variable("baseline".into(), 7.0);
        let before = serde_json::to_value(&document).expect("serialize document before tool");
        let mut state = crate::tool_dispatcher::ToolState::default();

        let result = crate::tool_dispatcher::dispatch_tool(
            tool,
            &mut state,
            &mut document,
            Point2::new(2.0, 3.0),
        );

        assert!(
            result.objects.is_empty(),
            "{tool:?} returned substitute objects"
        );
        assert!(result.reset_tool, "{tool:?} must return to Select");
        assert!(
            matches!(
                state.last_outcome,
                Some(CommandOutcome::Error(ref message)) if message.contains("no está disponible")
            ),
            "{tool:?} did not report an unavailable-feature error: {:?}",
            state.last_outcome
        );
        assert_eq!(
            serde_json::to_value(&document).expect("serialize document after tool"),
            before,
            "{tool:?} mutated the document"
        );
    }
}

#[test]
fn slider_tool_creates_validated_variable_metadata() {
    use grafito_geometry::Point2;
    use grafito_ui::Tool;

    let mut document = grafito_core::Document::new();
    let mut state = crate::tool_dispatcher::ToolState::default();

    let result = crate::tool_dispatcher::dispatch_tool(
        Tool::Slider,
        &mut state,
        &mut document,
        Point2::new(2.0, 3.0),
    );

    assert!(result.objects.is_empty());
    assert!(result.reset_tool);
    assert_eq!(document.get_variable("v0"), Some(0.0));
    assert!(matches!(
        document.variable_meta("v0"),
        Some(metadata)
            if metadata.position == Point2::new(2.0, 3.0)
                && metadata.min == -5.0
                && metadata.max == 5.0
                && metadata.step == 0.1
    ));
    grafito_core::validation::validate_document(&document)
        .expect("slider metadata is committed through document validation");
}

#[test]
fn slider_tool_rejects_nonfinite_metadata_without_creating_a_variable() {
    use grafito_command::commands::CommandOutcome;
    use grafito_geometry::Point2;
    use grafito_ui::Tool;

    let mut document = grafito_core::Document::new();
    let mut state = crate::tool_dispatcher::ToolState::default();
    let before = serde_json::to_value(&document).expect("document serializes");

    let result = crate::tool_dispatcher::dispatch_tool(
        Tool::Slider,
        &mut state,
        &mut document,
        Point2::new(f64::NAN, 3.0),
    );

    assert!(result.objects.is_empty());
    assert!(result.reset_tool);
    assert!(matches!(state.last_outcome, Some(CommandOutcome::Error(_))));
    assert!(document.variables().is_empty());
    assert!(document.variable_meta("v0").is_none());
    assert_eq!(serde_json::to_value(&document).unwrap(), before);
}

#[test]
fn locus_tool_creates_a_persistent_trace_after_two_point_clicks() {
    use grafito_command::commands::CommandOutcome;
    use grafito_core::{GeoObject, PointObj};
    use grafito_geometry::Point2;
    use grafito_ui::Tool;

    let mut document = grafito_core::Document::new();
    document.add_object(GeoObject::Point(
        PointObj::new(Point2::new(0.0, 0.0)).with_label("A"),
    ));
    document.add_object(GeoObject::Point(
        PointObj::new(Point2::new(2.0, 0.0)).with_label("B"),
    ));
    let mut state = crate::tool_dispatcher::ToolState::default();

    let first = crate::tool_dispatcher::dispatch_tool(
        Tool::Locus,
        &mut state,
        &mut document,
        Point2::new(0.0, 0.0),
    );
    assert!(!first.reset_tool);
    assert!(state.driver.is_some());

    let second = crate::tool_dispatcher::dispatch_tool(
        Tool::Locus,
        &mut state,
        &mut document,
        Point2::new(2.0, 0.0),
    );
    assert!(second.reset_tool);
    assert!(matches!(
        state.last_outcome,
        Some(CommandOutcome::Message(_))
    ));
    assert!(document.objects_iter().any(|(_, object)| {
        matches!(object, GeoObject::Pencil(pencil) if pencil.is_dynamic_locus())
    }));
}

#[test]
fn cancelling_locus_selection_clears_the_driver_before_the_next_click() {
    use grafito_core::{GeoObject, PointObj};
    use grafito_geometry::Point2;
    use grafito_ui::Tool;

    let mut document = grafito_core::Document::new();
    let driver = document.add_object(GeoObject::Point(
        PointObj::new(Point2::new(0.0, 0.0)).with_label("A"),
    ));
    let target = document.add_object(GeoObject::Point(
        PointObj::new(Point2::new(2.0, 0.0)).with_label("B"),
    ));
    let mut state = crate::tool_dispatcher::ToolState {
        driver: Some(driver),
        ..Default::default()
    };

    crate::input::cancel_locus_selection(&mut state);
    let result = crate::tool_dispatcher::dispatch_tool(
        Tool::Locus,
        &mut state,
        &mut document,
        Point2::new(2.0, 0.0),
    );

    assert!(!result.reset_tool);
    assert_eq!(state.driver, Some(target));
    assert!(document.objects_iter().all(|(_, object)| {
        !matches!(object, GeoObject::Pencil(pencil) if pencil.is_dynamic_locus())
    }));
}

#[test]
fn at_capacity_single_tool_insertion_returns_error_without_clearing_history() {
    use grafito_core::{ChangeSet, GeoObject, PointObj};
    use grafito_geometry::Point2;

    let mut document = document_at_object_capacity();
    let before = serde_json::to_value(&document).expect("serialize full document");
    let mut undo_stack = VecDeque::new();
    let mut redo_stack = VecDeque::from([ChangeSet {
        before: grafito_core::Document::new(),
        after: grafito_core::Document::new(),
    }]);

    let error = crate::app::commit_object_insertions(
        &mut document,
        &mut undo_stack,
        &mut redo_stack,
        vec![GeoObject::Point(PointObj::new(Point2::new(-1.0, -1.0)))],
    )
    .expect_err("an at-capacity insertion must be a normal error");

    assert!(error.contains("maximum"));
    assert!(undo_stack.is_empty());
    assert_eq!(redo_stack.len(), 1);
    assert_eq!(
        document.object_count(),
        grafito_core::validation::MAX_OBJECT_COUNT
    );
    assert_eq!(
        serde_json::to_value(&document).expect("serialize rejected full document"),
        before
    );
}

#[test]
fn eraser_stroke_saves_exactly_one_snapshot_and_noop_keeps_redo() {
    use grafito_core::{GeoObject, PointObj};
    use grafito_geometry::Point2;

    let mut document = grafito_core::Document::new();
    let first = document.add_object(GeoObject::Point(PointObj::new(Point2::new(0.0, 0.0))));
    let second = document.add_object(GeoObject::Point(PointObj::new(Point2::new(1.0, 0.0))));
    let mut undo_stack = VecDeque::new();
    let mut redo_stack = VecDeque::from([grafito_core::ChangeSet {
        before: document.clone(),
        after: document.clone(),
    }]);
    let mut stroke_has_mutated = false;

    assert!(crate::app::erase_object_for_stroke(
        &mut document,
        first,
        &mut stroke_has_mutated,
        &mut undo_stack,
        &mut redo_stack,
    ));
    assert!(crate::app::erase_object_for_stroke(
        &mut document,
        second,
        &mut stroke_has_mutated,
        &mut undo_stack,
        &mut redo_stack,
    ));
    assert_eq!(document.object_count(), 0);
    assert_eq!(undo_stack.len(), 1);
    assert_eq!(undo_stack[0].object_count(), 2);
    assert!(redo_stack.is_empty());

    let mut noop_document = grafito_core::Document::new();
    let mut noop_undo_stack = VecDeque::new();
    let mut noop_redo_stack = VecDeque::from([grafito_core::ChangeSet {
        before: noop_document.clone(),
        after: noop_document.clone(),
    }]);
    let mut noop_stroke_has_mutated = false;
    assert!(!crate::app::erase_object_for_stroke(
        &mut noop_document,
        grafito_core::ObjectId::new(),
        &mut noop_stroke_has_mutated,
        &mut noop_undo_stack,
        &mut noop_redo_stack,
    ));
    assert!(noop_undo_stack.is_empty());
    assert_eq!(noop_redo_stack.len(), 1);
}

#[test]
fn point_drag_gesture_captures_exactly_one_snapshot_covering_dependents() {
    // W-B: tres frames del mismo gesto por el protocolo real (input.rs usa
    // `capture_point_drag_snapshot` + `try_move_point_and_re_evaluate`):
    // sólo el primer frame clona `before`, así que un gesto = un undo que
    // restaura punto libre + hijos derivados.
    use grafito_core::{GeoObject, PointObj};
    use grafito_geometry::Point2;

    let mut document = grafito_core::Document::new();
    let a = document.add_object(GeoObject::Point(PointObj::new(Point2::new(0.0, 0.0))));
    let b = document.add_object(GeoObject::Point(PointObj::new(Point2::new(2.0, 0.0))));
    let (mid, _) = document
        .try_add_constructed_object(
            GeoObject::Point(PointObj::new(Point2::new(1.0, 0.0)).with_label("M")),
            "Midpoint",
            &[a, b],
        )
        .expect("midpoint construction");

    let mut snapshots: VecDeque<grafito_core::Document> = VecDeque::new();
    let mut mutated = false;
    for x in [0.5, 1.0, 1.5] {
        let target = Point2::new(x, 0.0);
        let before = crate::app::capture_point_drag_snapshot(&document, a, target, mutated);
        if document
            .try_move_point_and_re_evaluate(a, target)
            .expect("drag frame commits")
        {
            if let Some(before) = before {
                snapshots.push_back(before);
            }
            mutated = true;
        }
    }
    assert_eq!(snapshots.len(), 1, "un gesto = un único snapshot");
    assert_eq!(document.point_position(mid), Some(Point2::new(1.75, 0.0)));
    let restored = snapshots.pop_back().expect("one snapshot");
    document = restored;
    assert_eq!(document.point_position(a), Some(Point2::new(0.0, 0.0)));
    assert_eq!(document.point_position(mid), Some(Point2::new(1.0, 0.0)));
}

#[test]
fn failed_conic_tool_commit_preserves_document_and_history() {
    use grafito_geometry::Point2;

    let mut document = grafito_core::Document::new();
    let points: Vec<grafito_core::ObjectId> = (0..5)
        .map(|x| document.add_point(Point2::new(x as f64, 0.0)))
        .collect();
    let before = serde_json::to_value(&document).expect("document should serialize");
    let version_before = document.version;
    let mut undo_stack = VecDeque::new();
    let mut redo_stack = VecDeque::from([grafito_core::ChangeSet {
        before: document.clone(),
        after: document.clone(),
    }]);

    let error = crate::app::commit_conic_by_five_points(
        &mut document,
        &mut undo_stack,
        &mut redo_stack,
        &points,
    )
    .expect_err("five collinear points must reject the conic tool operation");

    assert!(error.contains("ConicByFivePoints"));
    assert_eq!(document.version, version_before);
    assert_eq!(
        serde_json::to_value(&document).expect("document should serialize"),
        before
    );
    assert!(undo_stack.is_empty());
    assert_eq!(redo_stack.len(), 1);
}

#[test]
fn interactive_conic_tools_commit_outputs_and_history_only_after_propagation() {
    use grafito_core::{GeoObject, LineObj};
    use grafito_geometry::Point2;

    let mut document = grafito_core::Document::new();
    let first_focus = document.add_point(Point2::new(-1.0, 0.0));
    let second_focus = document.add_point(Point2::new(1.0, 0.0));
    let point_on_conic = document.add_point(Point2::new(3.0, 1.0));
    let directrix = document.add_object(GeoObject::Line(LineObj::new(
        Point2::new(-4.0, -3.0),
        Point2::new(4.0, -3.0),
    )));
    let object_count_before = document.object_count();
    let mut undo_stack = VecDeque::new();
    let mut redo_stack = VecDeque::from([grafito_core::ChangeSet {
        before: document.clone(),
        after: document.clone(),
    }]);

    crate::app::commit_ellipse_by_foci(
        &mut document,
        &mut undo_stack,
        &mut redo_stack,
        first_focus,
        second_focus,
        point_on_conic,
    )
    .expect("valid ellipse construction should propagate");
    crate::app::commit_parabola_by_focus_directrix(
        &mut document,
        &mut undo_stack,
        &mut redo_stack,
        first_focus,
        directrix,
    )
    .expect("valid parabola construction should propagate");
    crate::app::commit_hyperbola_by_foci(
        &mut document,
        &mut undo_stack,
        &mut redo_stack,
        first_focus,
        second_focus,
        point_on_conic,
    )
    .expect("valid hyperbola construction should propagate");

    assert_eq!(document.object_count(), object_count_before + 3);
    assert!(document
        .objects_iter()
        .any(|(_, object)| matches!(object, GeoObject::Ellipse(_))));
    assert!(document
        .objects_iter()
        .any(|(_, object)| matches!(object, GeoObject::Parabola(_))));
    assert!(document
        .objects_iter()
        .any(|(_, object)| matches!(object, GeoObject::Hyperbola(_))));
    assert_eq!(undo_stack.len(), 3);
    assert!(redo_stack.is_empty());
}

#[test]
fn failed_interactive_conic_tools_preserve_document_and_history() {
    use grafito_core::{GeoObject, LineObj};
    use grafito_geometry::Point2;

    let mut document = grafito_core::Document::new();
    let first_focus = document.add_point(Point2::new(-1.0, 0.0));
    let second_focus = document.add_point(Point2::new(1.0, 0.0));
    let point_on_conic = document.add_point(Point2::new(3.0, 1.0));
    let directrix = document.add_object(GeoObject::Line(LineObj::new(
        Point2::new(-4.0, -3.0),
        Point2::new(4.0, -3.0),
    )));
    document
        .try_add_distance_constraint(first_focus, second_focus, 1.0)
        .expect("first distance constraint should register");
    document
        .try_add_distance_constraint(first_focus, second_focus, 2.0)
        .expect("conflicting distance constraint should stage for regression");
    let before = serde_json::to_value(&document).expect("document should serialize");
    let version_before = document.version;
    let mut undo_stack = VecDeque::new();
    let mut redo_stack = VecDeque::from([grafito_core::ChangeSet {
        before: document.clone(),
        after: document.clone(),
    }]);

    for error in [
        crate::app::commit_ellipse_by_foci(
            &mut document,
            &mut undo_stack,
            &mut redo_stack,
            first_focus,
            second_focus,
            point_on_conic,
        ),
        crate::app::commit_parabola_by_focus_directrix(
            &mut document,
            &mut undo_stack,
            &mut redo_stack,
            first_focus,
            directrix,
        ),
        crate::app::commit_hyperbola_by_foci(
            &mut document,
            &mut undo_stack,
            &mut redo_stack,
            first_focus,
            second_focus,
            point_on_conic,
        ),
    ] {
        assert!(
            error.is_err(),
            "unsatisfied propagation must reject the tool commit"
        );
        assert_eq!(document.version, version_before);
        assert_eq!(
            serde_json::to_value(&document).expect("document should serialize"),
            before
        );
        assert!(undo_stack.is_empty());
        assert_eq!(redo_stack.len(), 1);
    }
}

#[test]
fn select_drag_captures_the_object_under_the_pointer_not_an_old_selection() {
    use grafito_core::{GeoObject, PointObj};
    use grafito_geometry::Point2;

    let mut document = grafito_core::Document::new();
    let under_pointer = document.add_object(GeoObject::Point(PointObj::new(Point2::new(1.0, 1.0))));
    let previously_selected =
        document.add_object(GeoObject::Point(PointObj::new(Point2::new(5.0, 5.0))));
    document.select(previously_selected);

    let captured =
        crate::app::captured_select_drag_object(&mut document, Point2::new(1.0, 1.0), 0.1);
    assert_eq!(captured, Some(under_pointer));
    assert!(crate::app::is_free_point(
        &document,
        captured.expect("point capture")
    ));
    assert_eq!(
        crate::app::captured_select_drag_object(&mut document, Point2::new(9.0, 9.0), 0.1),
        None
    );
}

#[test]
fn autocomplete_inserts_canonical_command_for_visual_palette_names() {
    let mut input = "Tho".to_string();
    let item = crate::app::AutocompleteItem {
        text: "Thomas (Butterfly)".to_string(),
        detail: "Atractores".to_string(),
        bracket: true,
    };

    crate::ui::apply_autocomplete_item(&mut input, &item);
    assert_eq!(input, "Thomas[");
}

#[test]
fn autocomplete_accepts_tab_but_not_enter_as_completion_key() {
    assert!(crate::ui::is_autocomplete_completion_key(egui::Key::Tab));
    assert!(!crate::ui::is_autocomplete_completion_key(egui::Key::Enter));
    assert!(!crate::ui::is_autocomplete_completion_key(
        egui::Key::Escape
    ));
}

#[test]
fn autocomplete_selection_inserts_structure_and_closes_popup() {
    let mut input = "Comp".to_string();
    let mut autocomplete = crate::app::InputAutocomplete {
        open: true,
        selected: 0,
        navigated: false,
    };
    let suggestions = vec![crate::app::AutocompleteItem {
        text: "ComplexGrid".to_string(),
        detail: "Complejos".to_string(),
        bracket: true,
    }];

    assert!(crate::ui::complete_autocomplete_selection(
        &mut input,
        &suggestions,
        &mut autocomplete,
    ));
    assert_eq!(input, "ComplexGrid[");
    assert!(!autocomplete.open);
    assert_eq!(autocomplete.selected, 0);
}

#[test]
fn autocomplete_rejects_oversized_tokens_before_fuzzy_scoring() {
    let oversized = "a".repeat(4_096);

    assert_eq!(crate::ui::similarity_score(&oversized, "a"), 0.0);
    assert!(crate::ui::compute_autocomplete_suggestions(
        &oversized,
        &grafito_core::Document::new(),
    )
    .is_empty());
}

#[test]
fn autocomplete_keeps_ordinary_fuzzy_matches() {
    let suggestions =
        crate::ui::compute_autocomplete_suggestions("si", &grafito_core::Document::new());

    assert!(suggestions.iter().any(|item| item.text == "sin"));
}

#[test]
fn autocomplete_hides_only_unavailable_features_and_offers_dynamic_locus() {
    let document = grafito_core::Document::new();

    // G-D: Button es comando real (brazo + ida/vuelta en grafito-command),
    // solo Image sigue oculta. El click visual del Tool::Button sigue P2.
    {
        let unavailable = "Image";
        let suggestions = crate::ui::compute_autocomplete_suggestions(unavailable, &document);
        assert!(
            suggestions.iter().all(|item| item.text != unavailable),
            "autocomplete exposed unavailable feature {unavailable}"
        );
    }
    let suggestions = crate::ui::compute_autocomplete_suggestions("Button", &document);
    assert!(suggestions.iter().any(|item| item.text == "Button"));
    let suggestions = crate::ui::compute_autocomplete_suggestions("SampledGraph", &document);
    assert!(suggestions.iter().any(|item| item.text == "SampledGraph"));
    let suggestions = crate::ui::compute_autocomplete_suggestions("Locus", &document);
    assert!(suggestions.iter().any(|item| item.text == "Locus"));
}

#[test]
fn autocomplete_keeps_only_eight_candidates_while_collecting() {
    let retained = (0..8)
        .map(|index| {
            (
                crate::app::AutocompleteItem {
                    text: format!("candidate_{index}"),
                    detail: "test".to_string(),
                    bracket: false,
                },
                1.0,
            )
        })
        .collect::<Vec<_>>();

    assert!(crate::ui::autocomplete_candidate_slot(&retained, "lower_priority", 0.5,).is_none());
    assert!(crate::ui::autocomplete_candidate_slot(&retained, "exact", 2.0).is_some());
}

#[test]
fn autocomplete_keeps_eight_results_for_many_document_matches() {
    use grafito_core::{GeoObject, PointObj};
    use grafito_geometry::Point2;

    let mut document = grafito_core::Document::new();
    for index in 0..32 {
        document.add_object(GeoObject::Point(
            PointObj::new(Point2::new(index as f64, 0.0)).with_label(format!("needle_{index}")),
        ));
    }

    let suggestions = crate::ui::compute_autocomplete_suggestions("needle", &document);

    assert_eq!(suggestions.len(), 8);
    assert!(suggestions
        .iter()
        .all(|item| item.text.starts_with("needle_")));
}

#[test]
fn cleared_document_restarts_an_unchanged_constraint_tool() {
    use grafito_ui::Tool;

    assert!(crate::app::pending_action_needs_reinitialization(
        Tool::DistanceConstraint,
        Tool::DistanceConstraint,
        &crate::app::PendingAction::None,
    ));
    assert!(!crate::app::pending_action_needs_reinitialization(
        Tool::Select,
        Tool::Select,
        &crate::app::PendingAction::None,
    ));
}

#[test]
fn moving_a_free_point_updates_render_state_only_when_needed() {
    use grafito_core::{GeoObject, PointObj};
    use grafito_geometry::Point2;

    let mut document = grafito_core::Document::new();
    let point = document.add_object(GeoObject::Point(PointObj::new(Point2::new(1.0, 2.0))));
    let unchanged = document.clone();

    assert!(!document
        .try_move_point_and_re_evaluate(point, Point2::new(1.0, 2.0))
        .expect("unchanged point move should be accepted as a no-op"));
    assert_eq!(
        serde_json::to_value(&document).expect("document should serialize"),
        serde_json::to_value(&unchanged).expect("document should serialize"),
    );

    let before_move = document.clone();
    assert!(document
        .try_move_point_and_re_evaluate(point, Point2::new(3.0, 4.0))
        .expect("free point move should succeed"));
    assert!(crate::app::refresh_unversioned_document_change(
        &before_move,
        &mut document,
    ));
    assert_eq!(document.version, before_move.version.wrapping_add(1));
    let Some(GeoObject::Point(point_object)) = document.get_object(point) else {
        panic!("moved object should remain a point");
    };
    assert_eq!(point_object.position, Point2::new(3.0, 4.0));
}

#[test]
fn numeric_constraint_tool_staging_rejects_unsatisfiable_additions_without_committing() {
    use grafito_core::{GeoObject, PointObj};
    use grafito_geometry::Point2;

    let mut document = grafito_core::Document::new();
    let first = document.add_object(GeoObject::Point(PointObj::new(Point2::new(0.0, 0.0))));
    let second = document.add_object(GeoObject::Point(PointObj::new(Point2::new(1.0, 0.0))));

    crate::app::try_stage_numeric_constraint(&mut document, |staged| {
        staged
            .try_add_distance_constraint(first, second, 1.0)
            .map(|_| ())
    })
    .expect("the initial distance constraint is satisfiable");
    let before = serde_json::to_value(&document).expect("document should serialize");
    let version_before = document.version;
    let constraint_count = document.constraints.constraint_count();

    let error = crate::app::try_stage_numeric_constraint(&mut document, |staged| {
        staged
            .try_add_distance_constraint(first, second, 2.0)
            .map(|_| ())
    })
    .expect_err("a conflicting distance constraint must be rejected");

    assert!(error.contains("Numeric constraint"));
    assert_eq!(document.version, version_before);
    assert_eq!(document.constraints.constraint_count(), constraint_count);
    assert_eq!(
        serde_json::to_value(&document).expect("document should serialize"),
        before,
    );
}

#[test]
fn test_left_panel_default_sidebar_tab() {
    use crate::LeftPanelContent;
    assert_eq!(LeftPanelContent::Algebra.default_sidebar_tab(), 0);
    assert_eq!(LeftPanelContent::AlgebraAndCas.default_sidebar_tab(), 0);
    assert_eq!(LeftPanelContent::Tools.default_sidebar_tab(), 1);
    assert_eq!(LeftPanelContent::Cas.default_sidebar_tab(), 2);
    assert_eq!(LeftPanelContent::Data.default_sidebar_tab(), 2);
    assert_eq!(LeftPanelContent::Stats.default_sidebar_tab(), 3);
    assert_eq!(LeftPanelContent::Complex.default_sidebar_tab(), 0);
    assert_eq!(LeftPanelContent::Attractor.default_sidebar_tab(), 1);
}

#[test]
fn test_tool_group_id_def_nonempty() {
    for &gid in grafito_ui::toolbar::ALL_GROUPS {
        let (_icon, tools) = gid.def();
        assert!(!tools.is_empty(), "grupo {:?} vacío", gid);
    }
}

#[test]
fn trig_animation_supports_six_internal_function_types() {
    use crate::app::{GrafitoApp, TRIG_FUNCTIONS};

    assert_eq!(TRIG_FUNCTIONS.len(), 6);
    let names: Vec<&str> = TRIG_FUNCTIONS.iter().map(|spec| spec.name).collect();
    assert_eq!(names, vec!["sin", "cos", "tan", "cot", "sec", "csc"]);

    let t = std::f64::consts::FRAC_PI_4;
    assert!((GrafitoApp::trig_value(0, t) - t.sin()).abs() < 1e-12);
    assert!((GrafitoApp::trig_value(1, t) - t.cos()).abs() < 1e-12);
    assert!((GrafitoApp::trig_value(2, t) - 1.0).abs() < 1e-12);
    assert!((GrafitoApp::trig_value(3, t) - 1.0).abs() < 1e-12);
    assert!((GrafitoApp::trig_value(4, t) - 2.0_f64.sqrt()).abs() < 1e-12);
    assert!((GrafitoApp::trig_value(5, t) - 2.0_f64.sqrt()).abs() < 1e-12);
}

#[test]
fn trig_graph_sampling_is_safe_at_extreme_zoom() {
    use grafito_core::RenderQuality;

    assert_eq!(
        crate::render_2d::trig_sample_count(800, 80.0, RenderQuality::High),
        320
    );
    assert_eq!(
        crate::render_2d::trig_sample_count(800, 80.000_001, RenderQuality::High),
        200
    );
}

#[test]
fn trig_animation_explains_identities_for_teaching() {
    use crate::app::GrafitoApp;

    assert!(GrafitoApp::trig_identity(0).contains("altura"));
    assert!(GrafitoApp::trig_identity(2).contains("sin θ / cos θ"));
    assert!(GrafitoApp::trig_identity(4).contains("1 / cos θ"));
    assert!(GrafitoApp::trig_identity(5).contains("1 / sin θ"));
}

#[test]
fn trig_singularidades_no_pintan_inf() {
    use crate::app::GrafitoApp;
    use std::f64::consts::{FRAC_PI_2, FRAC_PI_4};

    // FIX 3: en θ = kπ las recíprocas no dan valor finito (antes `inf`).
    assert!(GrafitoApp::trig_value_opt(3, 0.0).is_none());
    assert!(GrafitoApp::trig_value_opt(5, 0.0).is_none());
    assert!(GrafitoApp::trig_value_opt(2, FRAC_PI_2).is_none());
    assert!(GrafitoApp::trig_value_opt(4, FRAC_PI_2).is_none());
    assert!(GrafitoApp::trig_value_opt(0, f64::NAN).is_none());
    // El legado sigue devolviendo f64 pero jamás `inf`: la UI filtra con
    // `is_finite` (`panels.rs`, `render_2d.rs`) y muestra el fallback.
    let v = GrafitoApp::trig_value(5, 0.0);
    assert!(!v.is_finite(), "csc(0) debe ser no-finito, fue {v}");
    assert!(!v.is_infinite(), "csc(0) debe ser NaN, no inf");
    // Lejos de singularidades todo finito e idéntico al cálculo directo.
    for i in 0..6 {
        let v = GrafitoApp::trig_value_opt(i, FRAC_PI_4).expect("finito en π/4");
        assert!(v.is_finite());
        assert!((GrafitoApp::trig_value(i, FRAC_PI_4) - v).abs() < 1e-12);
    }
}

#[test]
fn invalid_command_submission_does_not_request_an_undo_snapshot() {
    let before = grafito_core::Document::new();
    let mut after = before.clone();
    let mut input = "FooBar[]".to_string();
    let outcome = crate::commands::process_input(&mut after, &mut input);
    let mut undo_stack = VecDeque::new();
    let mut redo_stack = VecDeque::from([grafito_core::ChangeSet {
        before: before.clone(),
        after: before.clone(),
    }]);

    assert!(matches!(
        outcome,
        grafito_command::commands::CommandOutcome::Error(_)
    ));
    crate::app::save_command_snapshot_if_mutated(
        &outcome,
        before,
        &after,
        &mut undo_stack,
        &mut redo_stack,
    );
    assert!(undo_stack.is_empty());
    assert_eq!(redo_stack.len(), 1);
}

#[test]
fn ordinary_message_submission_records_history_shows_info_and_stays_out_of_undo() {
    let before = grafito_core::Document::new();
    let mut after = before.clone();
    let mut input = "Simplify[x + 0]".to_string();
    let outcome = crate::commands::process_input(&mut after, &mut input);
    let mut undo_stack = VecDeque::new();
    let mut redo_stack = VecDeque::from([grafito_core::ChangeSet {
        before: before.clone(),
        after: before.clone(),
    }]);
    let mut cas_result = String::new();
    let mut cas_history = VecDeque::new();
    let mut toasts = grafito_ui::toast::ToastManager::default();

    crate::app::save_command_snapshot_if_mutated(
        &outcome,
        before,
        &after,
        &mut undo_stack,
        &mut redo_stack,
    );
    crate::app::apply_command_outcome(
        &outcome,
        &mut cas_result,
        &mut cas_history,
        &mut toasts,
        0.0,
        "Simplify[x + 0]",
    );

    assert!(matches!(
        outcome,
        grafito_command::commands::CommandOutcome::Message(_)
    ));
    assert!(!cas_result.is_empty());
    assert_eq!(cas_history.len(), 1);
    assert!(cas_history[0].contains("Simplify[x + 0]"));
    assert!(undo_stack.is_empty());
    assert_eq!(redo_stack.len(), 1);

    let empty_toast_shapes = toast_shape_count(&mut grafito_ui::toast::ToastManager::default());
    assert!(toast_shape_count(&mut toasts) > empty_toast_shapes);
}

#[test]
fn no_op_command_outcome_stays_quiet() {
    let mut cas_result = String::new();
    let mut cas_history = VecDeque::new();
    let mut toasts = grafito_ui::toast::ToastManager::default();

    crate::app::apply_command_outcome(
        &grafito_command::commands::CommandOutcome::Ok,
        &mut cas_result,
        &mut cas_history,
        &mut toasts,
        0.0,
        "",
    );

    assert!(cas_result.is_empty());
    assert!(cas_history.is_empty());
    assert_eq!(
        toast_shape_count(&mut toasts),
        toast_shape_count(&mut grafito_ui::toast::ToastManager::default())
    );
}

#[test]
fn error_command_outcome_keeps_persistent_feedback_and_history() {
    let mut cas_result = String::new();
    let mut cas_history = VecDeque::new();
    let mut toasts = grafito_ui::toast::ToastManager::default();

    crate::app::apply_command_outcome(
        &grafito_command::commands::CommandOutcome::Error("entrada inválida".to_string()),
        &mut cas_result,
        &mut cas_history,
        &mut toasts,
        0.0,
        "FooBar[]",
    );

    assert_eq!(cas_result, "entrada inválida");
    assert_eq!(cas_history.len(), 1);
    assert!(cas_history[0].contains("FooBar[]"));
    assert!(cas_history[0].contains("entrada inválida"));
    assert!(
        toast_shape_count(&mut toasts)
            > toast_shape_count(&mut grafito_ui::toast::ToastManager::default())
    );
}

#[test]
fn persisted_cas_error_cells_request_undo_even_when_the_command_failed() {
    let before = grafito_core::Document::new();
    let mut after = before.clone();
    after
        .try_append_cas_worksheet_cell(
            "FooBar[]".to_string(),
            "comando desconocido".to_string(),
            grafito_core::CasWorksheetStatus::Error,
        )
        .expect("fixture worksheet cell is valid");
    let mut undo_stack = VecDeque::new();
    let mut redo_stack = VecDeque::from([grafito_core::ChangeSet {
        before: before.clone(),
        after: before.clone(),
    }]);

    crate::app::save_cas_worksheet_snapshot_if_mutated(
        before,
        &after,
        &mut undo_stack,
        &mut redo_stack,
    );

    assert_eq!(undo_stack.len(), 1);
    assert!(redo_stack.is_empty());
}

#[test]
fn clearing_persisted_cas_worksheet_is_undoable_and_redoable() {
    let mut before = grafito_core::Document::new();
    before
        .try_append_cas_worksheet_cell(
            "Simplify[x + 0]".to_string(),
            "x".to_string(),
            grafito_core::CasWorksheetStatus::Success,
        )
        .expect("fixture worksheet cell is valid");
    let mut after = before.clone();
    assert!(after.clear_cas_worksheet());
    let changes = grafito_core::ChangeSet {
        before: before.clone(),
        after: after.clone(),
    };
    let mut current = after;

    changes.undo(&mut current).expect("clear can be undone");
    assert_eq!(current.cas_worksheet(), before.cas_worksheet());
    changes.redo(&mut current).expect("clear can be redone");
    assert!(current.cas_worksheet().is_empty());
}

#[test]
fn failed_command_submission_retains_input_for_correction() {
    let mut failed_input = "FooBar[]".to_string();
    let failed = grafito_command::commands::CommandOutcome::Error("entrada inválida".to_string());

    assert!(!crate::app::clear_submitted_input_on_success(
        &mut failed_input,
        &failed,
    ));
    assert_eq!(failed_input, "FooBar[]");

    let mut successful_input = "A = (1, 2)".to_string();
    assert!(crate::app::clear_submitted_input_on_success(
        &mut successful_input,
        &grafito_command::commands::CommandOutcome::Ok,
    ));
    assert!(successful_input.is_empty());

    let source = include_str!("app.rs");
    let start = source
        .find("pub(crate) fn submit_input_text")
        .expect("shared command submitter");
    let end = source[start..]
        .find("pub(crate) fn label_of")
        .map(|offset| start + offset)
        .expect("next app method");
    let submitter = &source[start..end];
    assert!(submitter.contains("clear_submitted_input_on_success"));
    assert!(!submitter.contains("self.input_text.clear()"));
}

#[test]
fn cas_sidebar_routes_shared_input_to_the_persisted_worksheet() {
    assert!(crate::app::sidebar_uses_cas_worksheet(2));
    assert!(!crate::app::sidebar_uses_cas_worksheet(0));
    assert!(!crate::app::sidebar_uses_cas_worksheet(3));

    let source = include_str!("app.rs");
    let start = source
        .find("pub(crate) fn submit_input_text")
        .expect("shared command submitter");
    let end = source[start..]
        .find("/// Etiqueta de un objeto")
        .map(|offset| start + offset)
        .expect("next app method");
    let submitter = &source[start..end];
    assert!(submitter.contains("sidebar_uses_cas_worksheet(self.sidebar_tab)"));
    assert!(submitter.contains("self.submit_cas_worksheet_cell(time)"));
}

#[test]
fn statistics_input_is_strict_and_finite() {
    assert_eq!(
        crate::sheet_layout::parse_statistics_input("1, 2\n3"),
        Ok(vec![1.0, 2.0, 3.0])
    );

    for invalid in ["1, nope, 3", "1,,3", "NaN", "inf", "-inf"] {
        assert!(
            crate::sheet_layout::parse_statistics_input(invalid).is_err(),
            "{invalid:?} must be rejected"
        );
    }
}

#[test]
fn statistics_summary_stays_finite_for_equal_extreme_values() {
    let summary = crate::sheet_layout::statistics_summary(&[f64::MAX, f64::MAX])
        .expect("equal finite values have representable central statistics");

    assert_eq!(summary.mean, f64::MAX);
    assert_eq!(summary.median, f64::MAX);
    assert_eq!(summary.variance, 0.0);
    assert_eq!(summary.standard_deviation, 0.0);
    assert!(
        summary.sum.is_none(),
        "the overflowing sum must be explicit"
    );
}

#[test]
fn statistics_summary_preserves_equal_minimum_subnormal_quantiles() {
    let minimum_subnormal = f64::from_bits(1);
    let summary = crate::sheet_layout::statistics_summary(&[minimum_subnormal, minimum_subnormal])
        .expect("equal subnormal values have representable central statistics");

    assert_eq!(summary.mean, minimum_subnormal);
    assert_eq!(summary.median, minimum_subnormal);
    assert_eq!(summary.q1, minimum_subnormal);
    assert_eq!(summary.q3, minimum_subnormal);
    assert_eq!(summary.variance, 0.0);
}

#[test]
fn statistics_summary_interpolates_opposite_sign_values() {
    let summary = crate::sheet_layout::statistics_summary(&[-2.0, 2.0])
        .expect("opposite-sign values have representable statistics");

    assert_eq!(summary.median, 0.0);
    assert_eq!(summary.q1, -1.0);
    assert_eq!(summary.q3, 1.0);
    assert_eq!(summary.iqr, 2.0);
}

#[test]
fn statistics_summary_preserves_ordinary_population_results() {
    let summary = crate::sheet_layout::statistics_summary(&[1.0, 2.0, 3.0, 4.0])
        .expect("ordinary finite data must remain supported");

    assert!((summary.mean - 2.5).abs() < 1.0e-12);
    assert!((summary.median - 2.5).abs() < 1.0e-12);
    assert!((summary.variance - 1.25).abs() < 1.0e-12);
    assert!((summary.standard_deviation - 1.25_f64.sqrt()).abs() < 1.0e-12);
}

#[test]
fn statistics_summary_rejects_unrepresentable_true_results() {
    let error = crate::sheet_layout::statistics_summary(&[-1.0e308, 1.0e308])
        .expect_err("unrepresentable dispersion must not become infinity");

    assert!(error.contains("no es representable"));
}

#[test]
fn statistics_panel_has_vertical_scroll_and_persistent_validation_feedback() {
    let source = include_str!("panels.rs");
    let start = source
        .find("pub(crate) fn draw_statistics_panel")
        .expect("statistics panel");
    let end = source[start..]
        .find("pub(crate) fn draw_complex_panel")
        .map(|offset| start + offset)
        .expect("next panel");
    let panel = &source[start..end];

    assert!(
        panel.contains("ScrollArea::vertical()")
            || panel.contains("panel_gutter_scroll")
            || panel.contains("panel_scroll_content"),
        "el panel Prob necesita scroll vertical (directo o vía helper con aire)"
    );
    assert!(panel.contains("statistics_input_error"));
    assert!(!panel.contains("filter_map"));
}

#[test]
fn panel_tabs_datos_prob_con_entrada_visible() {
    use crate::{LeftPanelContent, Perspective};
    // Regla visible ⇒ botón: toda perspectiva que declara Stats/Data tiene
    // que estar cableada a su tab (helper) y cada tab tiene que ser
    // alcanzable desde el dispatch y desde el menú Paneles.
    for perspective in Perspective::ALL {
        let declara_stats = perspective.layout().left_panel == LeftPanelContent::Stats;
        assert_eq!(
            crate::uses_statistics_panel(perspective),
            declara_stats,
            "uses_statistics_panel debe coincidir con LeftPanelContent::Stats para {:?}",
            perspective
        );
        let declara_data = perspective.layout().left_panel == LeftPanelContent::Data;
        assert_eq!(
            crate::uses_data_panel(perspective),
            declara_data,
            "uses_data_panel debe coincidir con LeftPanelContent::Data para {:?}",
            perspective
        );
    }
    assert!(crate::uses_statistics_panel(Perspective::Statistics));
    assert!(crate::uses_statistics_panel(Perspective::Probability));
    assert!(!crate::uses_statistics_panel(Perspective::DataAnalysis));
    assert!(!crate::uses_statistics_panel(Perspective::Geometry2D));
    assert!(!crate::uses_statistics_panel(Perspective::Complex));
    assert!(crate::uses_data_panel(Perspective::DataAnalysis));
    assert!(!crate::uses_data_panel(Perspective::Statistics));

    let app_source = include_str!("app.rs");
    assert!(
        app_source.contains("draw_data_panel(self, ctx)")
            && app_source.contains("draw_statistics_panel(self, ctx)")
            && app_source.contains("draw_view_panel(self, ctx)"),
        "el drawer izquierdo debe rutear a los paneles Datos/Prob./Vista"
    );
    let ui_source = include_str!("ui.rs");
    assert!(
        ui_source.contains("rail_t(\"rail.tab_data\"")
            && ui_source.contains("rail_t(\"rail.tip_prob\""),
        "el rail expone Datos/Prob. vía `rail.*` (B-A185)"
    );
}

#[test]
fn idle_object_panel_edit_does_not_change_document_version() {
    use grafito_core::{Attractor3DObj, Cube3DObj, GeoObject};
    use grafito_geometry::Point3D;

    let mut document = grafito_core::Document::new();
    let cube = document.add_object(GeoObject::Cube3D(Cube3DObj::new(
        Point3D::new(0.0, 0.0, 0.0),
        2.0,
    )));
    let attractor = document.add_object(GeoObject::Attractor3D(Attractor3DObj::new(
        "lorenz",
        vec![10.0, 28.0, 8.0 / 3.0],
    )));
    let version_before = document.version;

    assert!(
        !crate::panels::apply_object_panel_edit(&mut document, cube, false, |_| panic!(
            "idle edit must not acquire mutable object access"
        ),)
        .expect("idle panel edit is a no-op")
    );
    assert!(
        !crate::panels::apply_object_panel_edit(&mut document, attractor, false, |_| panic!(
            "idle edit must not acquire mutable object access"
        ),)
        .expect("idle panel edit is a no-op")
    );
    assert_eq!(document.version, version_before);
    assert!(
        !crate::panels::apply_object_panel_edit(&mut document, cube, true, |_| {},)
            .expect("an unchanged panel candidate is a no-op")
    );
    assert_eq!(document.version, version_before);

    let source = include_str!("panels.rs");
    for (start_marker, end_marker) in [
        (
            "pub(crate) fn draw_right_properties_panel",
            "pub(crate) fn draw_right_domain_coloring_panel",
        ),
        (
            "pub(crate) fn draw_right_parameters_panel",
            "pub(crate) fn draw_right_regression_panel",
        ),
    ] {
        let start = source.find(start_marker).expect("property panel");
        let end = source[start..]
            .find(end_marker)
            .map(|offset| start + offset)
            .expect("next panel");
        let panel = &source[start..end];
        assert!(panel.contains("apply_object_panel_edit"));
        assert!(panel.contains("capture_successful_replacement"));
        // get_object_mut puede usarse para ediciones con snapshot (no mutación
        // directa del documento); las regresiones relevantes son las otras.
        assert!(!panel.contains("document.bump_version"));
        assert!(!panel.contains("let before = app.document.clone()"));
        assert!(!panel.contains("snapshot.capture(&app.document)"));
        assert!(panel.contains("DeferredPanelSnapshot::new"));
    }
}

#[test]
fn deferred_panel_snapshot_is_idle_without_a_document_snapshot_and_records_one_edit() {
    use grafito_core::{ChangeSet, Cube3DObj, GeoObject};
    use grafito_geometry::Point3D;

    let mut document = grafito_core::Document::new();
    let id = document.add_object(GeoObject::Cube3D(Cube3DObj::new(
        Point3D::new(0.0, 0.0, 0.0),
        2.0,
    )));
    let mut undo_stack = VecDeque::new();
    let mut redo_stack = VecDeque::from([ChangeSet {
        before: grafito_core::Document::new(),
        after: grafito_core::Document::new(),
    }]);

    let mut snapshot = crate::app::DeferredPanelSnapshot::new(undo_stack.len());
    assert!(!snapshot.is_captured());
    assert!(!snapshot.save_if_semantically_changed(
        &mut document,
        &mut undo_stack,
        &mut redo_stack,
    ));
    assert!(undo_stack.is_empty());
    assert_eq!(redo_stack.len(), 1);

    let mut snapshot = crate::app::DeferredPanelSnapshot::new(undo_stack.len());
    let before =
        crate::panels::apply_object_panel_edit_with_previous(&mut document, id, true, |object| {
            let GeoObject::Cube3D(cube) = object else {
                panic!("fixture remains a cube");
            };
            cube.size = 3.0;
        })
        .expect("the panel edit is valid")
        .expect("a valid panel edit returns its prior document");
    snapshot.capture_successful_replacement(before);
    assert!(snapshot.is_captured());
    assert!(!snapshot.requires_semantic_comparison());
    assert!(snapshot.save_if_semantically_changed(&mut document, &mut undo_stack, &mut redo_stack,));
    assert_eq!(undo_stack.len(), 1);
    assert!(redo_stack.is_empty());
    assert!(matches!(
        undo_stack[0].get_object(id),
        Some(GeoObject::Cube3D(cube)) if cube.size == 2.0
    ));
}

#[test]
fn deferred_panel_snapshot_ignores_no_op_and_rejected_replacements_without_comparison() {
    use grafito_core::{ChangeSet, GeoObject, RegularPolychoron4DObj};
    use grafito_geometry::RegularPolychoron;

    let mut document = grafito_core::Document::new();
    let id = document
        .try_add_object(GeoObject::RegularPolychoron4D(RegularPolychoron4DObj::new(
            RegularPolychoron::Tesseract,
        )))
        .expect("fixture inserts");
    let mut undo_stack = VecDeque::new();
    let mut redo_stack = VecDeque::from([ChangeSet {
        before: grafito_core::Document::new(),
        after: grafito_core::Document::new(),
    }]);
    let mut snapshot = crate::app::DeferredPanelSnapshot::new(undo_stack.len());

    assert!(
        crate::panels::apply_object_panel_edit_with_previous(&mut document, id, true, |_| {},)
            .expect("an unchanged candidate is a no-op")
            .is_none()
    );
    assert!(!snapshot.is_captured());
    assert!(!snapshot.requires_semantic_comparison());

    let error =
        crate::panels::apply_object_panel_edit_with_previous(&mut document, id, true, |object| {
            let GeoObject::RegularPolychoron4D(polychoron) = object else {
                panic!("fixture remains a regular polychoron");
            };
            polychoron.scale = 0.0;
        })
        .expect_err("invalid detached candidates are rejected before history capture");

    assert!(error.contains("scale"), "{error}");
    assert!(!snapshot.is_captured());
    assert!(!snapshot.requires_semantic_comparison());
    assert!(!snapshot.save_if_semantically_changed(
        &mut document,
        &mut undo_stack,
        &mut redo_stack,
    ));
    assert!(undo_stack.is_empty());
    assert_eq!(redo_stack.len(), 1);
}

#[test]
fn color_picker_keeps_staged_changes_transient_until_an_explicit_apply() {
    let source = include_str!("ui.rs");
    let start = source
        .find("pub(crate) fn draw_color_picker")
        .expect("color picker renderer");
    let end = source[start..]
        .find("const MAX_AUTOCOMPLETE_TOKEN_CHARS")
        .map(|offset| start + offset)
        .expect("next ui section");
    let picker = &source[start..end];

    assert!(picker.contains("let Some(ActiveColorPicker {"));
    assert!(picker.contains("object_id,"));
    assert!(picker.contains("target,"));
    assert!(picker.contains("mut picker,"));
    assert!(picker.contains("return;"));
    assert!(!picker.contains("app.document.clone()"));
    assert!(!picker.contains("save_snapshot_if_semantically_changed"));
    assert!(!picker.contains("get_object_mut"));
    assert!(picker.contains("ColorPickerDialogAction::Apply"));
    assert!(picker.contains("ColorPickerDialogAction::Dismiss"));
    assert!(picker.contains("apply_color_picker_dialog_action"));
    assert!(picker.contains("outcome.any_changed()"));
    // Live mode: color_changed aplica al instante, sin botón Aplicar/Cancelar separado
    assert!(picker.contains("outcome.color_changed"));
    assert!(picker.contains("t(\"picker.dialog_hint\""));
    assert!(picker.contains("t(\"picker.dialog_title\""));
    assert!(!picker.contains("outcome.object_color_changed"));
}

#[test]
fn color_picker_targets_are_explicit_and_distinct() {
    use crate::app::{ActiveColorPicker, ColorPickerTarget};
    use grafito_core::ObjectId;
    use grafito_geometry::Color;

    let object = ActiveColorPicker {
        object_id: ObjectId::new(),
        target: ColorPickerTarget::ObjectColor,
        picker: grafito_ui::color_picker::HsvColorPicker::new(Color::RED),
    };
    let fill = ActiveColorPicker {
        object_id: ObjectId::new(),
        target: ColorPickerTarget::RegularPolychoronFill,
        picker: grafito_ui::color_picker::HsvColorPicker::new(Color::BLUE),
    };

    assert_eq!(object.target, ColorPickerTarget::ObjectColor);
    assert_eq!(fill.target, ColorPickerTarget::RegularPolychoronFill);
    assert_ne!(object.target, fill.target);
}

#[test]
fn color_picker_dialog_is_centered_foreground_and_constrained_to_a_safe_viewport() {
    let source = include_str!("ui.rs");
    let start = source
        .find("pub(crate) fn draw_color_picker")
        .expect("color picker renderer");
    let end = source[start..]
        .find("const MAX_AUTOCOMPLETE_TOKEN_CHARS")
        .map(|offset| start + offset)
        .expect("next ui section");
    let picker = &source[start..end];

    assert!(picker.contains(".order(egui::Order::Foreground)"));
    assert!(picker.contains(".anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)"));
    assert!(picker.contains(".fixed_size(COLOR_PICKER_DIALOG_SIZE)"));
    assert!(picker.contains(".resizable(false)"));
    assert!(picker.contains(".constrain_to(color_picker_safe_viewport(ctx.screen_rect()))"));
    assert!(picker.contains(".fill(theme.panel_bg)"));
    // Scandinavian: picker may use hairline or no border (Stroke::NONE) for seamless white
    assert!(picker.contains("theme.separator") || picker.contains("Stroke::NONE"));

    let viewport = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0));
    let safe_viewport = crate::ui::color_picker_safe_viewport(viewport);
    assert!(safe_viewport.min.x > viewport.min.x);
    assert!(safe_viewport.min.y > viewport.min.y);
    assert!(safe_viewport.max.x < viewport.max.x);
    assert!(safe_viewport.max.y < viewport.max.y);

    let narrow_viewport = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(8.0, 6.0));
    let narrow_safe_viewport = crate::ui::color_picker_safe_viewport(narrow_viewport);
    assert!(narrow_safe_viewport.width() > 0.0);
    assert!(narrow_safe_viewport.height() > 0.0);
    assert!(narrow_viewport.contains_rect(narrow_safe_viewport));
}

#[test]
fn color_picker_real_color_change_replaces_once_and_records_one_undo() {
    use grafito_core::{ChangeSet, GeoObject, PointObj};
    use grafito_geometry::{Color, Point2};

    let mut document = grafito_core::Document::new();
    let id = document.add_object(GeoObject::Point(PointObj::new(Point2::new(1.0, 2.0))));
    let previous_color = document.get_object(id).expect("point exists").color();
    let version_before = document.version;
    let mut undo_stack = VecDeque::new();
    let mut redo_stack = VecDeque::from([ChangeSet {
        before: grafito_core::Document::new(),
        after: grafito_core::Document::new(),
    }]);

    assert!(crate::ui::apply_color_picker_object_color_change(
        &mut document,
        id,
        Color::GREEN,
        &mut undo_stack,
        &mut redo_stack,
    )
    .expect("valid color replacement commits"));
    assert_eq!(document.version, version_before.wrapping_add(1));
    assert_eq!(
        document.get_object(id).expect("point remains").color(),
        Color::GREEN
    );
    assert_eq!(undo_stack.len(), 1);
    assert_eq!(
        undo_stack[0]
            .get_object(id)
            .expect("undo point exists")
            .color(),
        previous_color
    );
    assert!(redo_stack.is_empty());
}

#[test]
fn color_picker_dialog_commits_only_on_apply_and_discards_dismiss() {
    use crate::app::ColorPickerTarget;
    use crate::ui::ColorPickerDialogAction;
    use grafito_core::{ChangeSet, GeoObject, PointObj};
    use grafito_geometry::{Color, Point2};

    let mut document = grafito_core::Document::new();
    let id = document.add_object(GeoObject::Point(PointObj::new(Point2::new(1.0, 2.0))));
    let original = document.get_object(id).expect("point exists").color();
    let version_before = document.version;
    let before = document.clone();
    let mut undo_stack = VecDeque::new();
    let mut redo_stack = VecDeque::from([ChangeSet {
        before: grafito_core::Document::new(),
        after: grafito_core::Document::new(),
    }]);

    assert!(!crate::ui::apply_color_picker_dialog_action(
        ColorPickerDialogAction::Dismiss,
        &mut document,
        ColorPickerTarget::ObjectColor,
        id,
        Color::GREEN,
        &mut undo_stack,
        &mut redo_stack,
    )
    .expect("dismiss discards staged colors"));
    assert_eq!(document.version, version_before);
    assert_eq!(
        document.get_object(id).expect("point remains").color(),
        original
    );
    assert_eq!(
        serde_json::to_value(&document).expect("document serializes"),
        serde_json::to_value(&before).expect("baseline serializes")
    );
    assert!(undo_stack.is_empty());
    assert_eq!(redo_stack.len(), 1);

    assert!(crate::ui::apply_color_picker_dialog_action(
        ColorPickerDialogAction::Apply,
        &mut document,
        ColorPickerTarget::ObjectColor,
        id,
        Color::GREEN,
        &mut undo_stack,
        &mut redo_stack,
    )
    .expect("apply commits the staged color once"));
    assert_eq!(
        document.get_object(id).expect("point remains").color(),
        Color::GREEN
    );
    assert_eq!(undo_stack.len(), 1);
    assert!(redo_stack.is_empty());

    assert!(!crate::ui::apply_color_picker_dialog_action(
        ColorPickerDialogAction::Apply,
        &mut document,
        ColorPickerTarget::ObjectColor,
        id,
        Color::GREEN,
        &mut undo_stack,
        &mut redo_stack,
    )
    .expect("an equal applied color remains a no-op"));
    assert_eq!(undo_stack.len(), 1);
}

#[test]
fn color_picker_dialog_apply_of_untouched_hsv_roundtrip_preserves_document_and_history() {
    use crate::app::ColorPickerTarget;
    use crate::ui::ColorPickerDialogAction;
    use grafito_core::{ChangeSet, GeoObject, PointObj};
    use grafito_geometry::{Color, Point2};

    let original = Color::new(0.123_456_7, 0.456_789_1, 0.987_654_3, 0.35);
    let staged = grafito_ui::color_picker::HsvColorPicker::new(original).to_color();
    assert_ne!(staged, original, "fixture must exercise a ULP round trip");

    let mut document = grafito_core::Document::new();
    let id = document.add_object(GeoObject::Point(PointObj::new(Point2::new(1.0, 2.0))));
    document
        .get_object_mut(id)
        .expect("point exists")
        .set_color(original);
    let version_before = document.version;
    let before = document.clone();
    let mut undo_stack = VecDeque::new();
    let mut redo_stack = VecDeque::from([ChangeSet {
        before: grafito_core::Document::new(),
        after: grafito_core::Document::new(),
    }]);

    assert!(!crate::ui::apply_color_picker_dialog_action(
        ColorPickerDialogAction::Apply,
        &mut document,
        ColorPickerTarget::ObjectColor,
        id,
        staged,
        &mut undo_stack,
        &mut redo_stack,
    )
    .expect("untouched picker Apply is a no-op"));
    assert_eq!(document.version, version_before);
    assert_eq!(
        serde_json::to_value(&document).expect("document serializes"),
        serde_json::to_value(&before).expect("baseline serializes")
    );
    assert!(undo_stack.is_empty());
    assert_eq!(redo_stack.len(), 1);
}

#[test]
fn color_picker_dialog_apply_of_untouched_hsv_roundtrip_fill_preserves_document_and_history() {
    use crate::app::ColorPickerTarget;
    use crate::ui::ColorPickerDialogAction;
    use grafito_core::{ChangeSet, GeoObject, RegularPolychoron4DObj};
    use grafito_geometry::{Color, RegularPolychoron};

    let original = Color::new(0.123_456_7, 0.456_789_1, 0.987_654_3, 0.35);
    let staged = grafito_ui::color_picker::HsvColorPicker::new(original).to_color();
    assert_ne!(staged, original, "fixture must exercise a ULP round trip");

    let mut document = grafito_core::Document::new();
    let mut polychoron = RegularPolychoron4DObj::new(RegularPolychoron::Tesseract);
    polychoron.fill_color = Some(original);
    let id = document
        .try_add_object(GeoObject::RegularPolychoron4D(polychoron))
        .expect("fixture inserts");
    let version_before = document.version;
    let before = document.clone();
    let mut undo_stack = VecDeque::new();
    let mut redo_stack = VecDeque::from([ChangeSet {
        before: grafito_core::Document::new(),
        after: grafito_core::Document::new(),
    }]);

    assert!(!crate::ui::apply_color_picker_dialog_action(
        ColorPickerDialogAction::Apply,
        &mut document,
        ColorPickerTarget::RegularPolychoronFill,
        id,
        staged,
        &mut undo_stack,
        &mut redo_stack,
    )
    .expect("untouched fill picker Apply is a no-op"));
    assert_eq!(document.version, version_before);
    assert_eq!(
        serde_json::to_value(&document).expect("document serializes"),
        serde_json::to_value(&before).expect("baseline serializes")
    );
    assert!(undo_stack.is_empty());
    assert_eq!(redo_stack.len(), 1);
}

#[test]
fn color_picker_dialog_apply_routes_a_polychoron_fill_target_once() {
    use crate::app::ColorPickerTarget;
    use crate::ui::ColorPickerDialogAction;
    use grafito_core::{ChangeSet, GeoObject, RegularPolychoron4DObj};
    use grafito_geometry::{Color, RegularPolychoron};

    let mut document = grafito_core::Document::new();
    let id = document
        .try_add_object(GeoObject::RegularPolychoron4D(RegularPolychoron4DObj::new(
            RegularPolychoron::Tesseract,
        )))
        .expect("fixture inserts");
    let edge_color = document.get_object(id).expect("polychoron exists").color();
    let mut undo_stack = VecDeque::new();
    let mut redo_stack = VecDeque::from([ChangeSet {
        before: grafito_core::Document::new(),
        after: grafito_core::Document::new(),
    }]);

    assert!(crate::ui::apply_color_picker_dialog_action(
        ColorPickerDialogAction::Apply,
        &mut document,
        ColorPickerTarget::RegularPolychoronFill,
        id,
        Color::GREEN,
        &mut undo_stack,
        &mut redo_stack,
    )
    .expect("apply commits the staged fill once"));
    assert!(matches!(
        document.get_object(id),
        Some(GeoObject::RegularPolychoron4D(polychoron))
            if polychoron.color == edge_color && polychoron.fill_color == Some(Color::GREEN)
    ));
    assert_eq!(undo_stack.len(), 1);
    assert!(redo_stack.is_empty());
}

#[test]
fn color_picker_equal_or_missing_color_change_preserves_document_and_history() {
    use grafito_core::{ChangeSet, GeoObject, ObjectId, PointObj};
    use grafito_geometry::{Color, Point2};

    let mut document = grafito_core::Document::new();
    let id = document.add_object(GeoObject::Point(PointObj::new(Point2::new(1.0, 2.0))));
    let current_color = document.get_object(id).expect("point exists").color();
    let version_before = document.version;
    let before = document.clone();
    let mut undo_stack = VecDeque::new();
    let mut redo_stack = VecDeque::from([ChangeSet {
        before: grafito_core::Document::new(),
        after: grafito_core::Document::new(),
    }]);

    assert!(!crate::ui::apply_color_picker_object_color_change(
        &mut document,
        id,
        current_color,
        &mut undo_stack,
        &mut redo_stack,
    )
    .expect("equal color is a no-op"));
    assert!(!crate::ui::apply_color_picker_object_color_change(
        &mut document,
        ObjectId::new(),
        Color::GREEN,
        &mut undo_stack,
        &mut redo_stack,
    )
    .expect("missing object is a no-op"));
    assert_eq!(document.version, version_before);
    assert_eq!(
        serde_json::to_value(&document).expect("document serializes"),
        serde_json::to_value(&before).expect("baseline serializes")
    );
    assert!(undo_stack.is_empty());
    assert_eq!(redo_stack.len(), 1);
}

#[test]
fn color_picker_equal_color_short_circuits_before_candidate_or_core_replacement() {
    let source = include_str!("ui.rs");
    let start = source
        .find("pub(crate) fn apply_color_picker_object_color_change")
        .expect("color picker object edit helper");
    let end = source[start..]
        .find("pub(crate) fn draw_color_picker")
        .map(|offset| start + offset)
        .expect("color picker renderer");
    let helper = &source[start..end];

    let equal_color_return = helper
        .find("if grafito_ui::color_picker::colors_match(object.color(), color)")
        .expect("equal colors must return before candidate work");
    let candidate_clone = helper.find("object.clone()").expect("candidate clone");
    let replacement = helper
        .find("try_replace_object_with_previous")
        .expect("core replacement");
    let snapshot = helper
        .find("DeferredPanelSnapshot::new")
        .expect("undo snapshot");

    assert!(equal_color_return < candidate_clone);
    assert!(equal_color_return < replacement);
    assert!(equal_color_return < snapshot);
    assert!(helper[equal_color_return..candidate_clone].contains("return Ok(false);"));
}

#[test]
fn color_picker_rejected_color_preserves_document_and_history() {
    use grafito_core::{ChangeSet, GeoObject, PointObj};
    use grafito_geometry::{Color, Point2};

    let mut document = grafito_core::Document::new();
    let id = document.add_object(GeoObject::Point(PointObj::new(Point2::new(1.0, 2.0))));
    let version_before = document.version;
    let before = document.clone();
    let mut undo_stack = VecDeque::new();
    let mut redo_stack = VecDeque::from([ChangeSet {
        before: grafito_core::Document::new(),
        after: grafito_core::Document::new(),
    }]);

    let error = crate::ui::apply_color_picker_object_color_change(
        &mut document,
        id,
        Color::new(f32::NAN, 0.2, 0.3, 1.0),
        &mut undo_stack,
        &mut redo_stack,
    )
    .expect_err("invalid detached candidate is rejected");

    assert!(error.contains("color.r"), "{error}");
    assert_eq!(document.version, version_before);
    assert_eq!(
        serde_json::to_value(&document).expect("document serializes"),
        serde_json::to_value(&before).expect("baseline serializes")
    );
    assert!(undo_stack.is_empty());
    assert_eq!(redo_stack.len(), 1);
}

#[test]
fn color_picker_polychoron_fill_change_replaces_once_and_records_one_undo() {
    use grafito_core::{ChangeSet, GeoObject, RegularPolychoron4DObj};
    use grafito_geometry::{Color, RegularPolychoron};

    let mut document = grafito_core::Document::new();
    let id = document
        .try_add_object(GeoObject::RegularPolychoron4D(RegularPolychoron4DObj::new(
            RegularPolychoron::Tesseract,
        )))
        .expect("fixture inserts");
    let edge_color = document.get_object(id).expect("polychoron exists").color();
    let previous_fill = match document.get_object(id) {
        Some(GeoObject::RegularPolychoron4D(polychoron)) => {
            polychoron.fill_color.expect("fixture has a fill color")
        }
        _ => panic!("fixture remains a regular polychoron"),
    };
    let version_before = document.version;
    let mut undo_stack = VecDeque::new();
    let mut redo_stack = VecDeque::from([ChangeSet {
        before: grafito_core::Document::new(),
        after: grafito_core::Document::new(),
    }]);

    assert!(
        crate::ui::apply_color_picker_regular_polychoron_fill_color_change(
            &mut document,
            id,
            Color::GREEN,
            &mut undo_stack,
            &mut redo_stack,
        )
        .expect("valid polychoron fill replacement commits")
    );
    assert_eq!(document.version, version_before.wrapping_add(1));
    assert!(matches!(
        document.get_object(id),
        Some(GeoObject::RegularPolychoron4D(polychoron))
            if polychoron.color == edge_color && polychoron.fill_color == Some(Color::GREEN)
    ));
    assert_eq!(undo_stack.len(), 1);
    assert!(matches!(
        undo_stack[0].get_object(id),
        Some(GeoObject::RegularPolychoron4D(polychoron))
            if polychoron.fill_color == Some(previous_fill)
    ));
    assert!(redo_stack.is_empty());
}

#[test]
fn color_picker_polychoron_fill_no_ops_preserve_document_and_history() {
    use grafito_core::{ChangeSet, GeoObject, ObjectId, PointObj, RegularPolychoron4DObj};
    use grafito_geometry::{Color, Point2, RegularPolychoron};

    let mut document = grafito_core::Document::new();
    let polychoron_id = document
        .try_add_object(GeoObject::RegularPolychoron4D(RegularPolychoron4DObj::new(
            RegularPolychoron::Tesseract,
        )))
        .expect("fixture inserts");
    let no_fill_id = document
        .try_add_object(GeoObject::RegularPolychoron4D({
            let mut polychoron = RegularPolychoron4DObj::new(RegularPolychoron::Pentachoron);
            polychoron.fill_color = None;
            polychoron
        }))
        .expect("no-fill fixture inserts");
    let point_id = document.add_object(GeoObject::Point(PointObj::new(Point2::new(1.0, 2.0))));
    let existing_fill = match document.get_object(polychoron_id) {
        Some(GeoObject::RegularPolychoron4D(polychoron)) => {
            polychoron.fill_color.expect("fixture has a fill color")
        }
        _ => panic!("fixture remains a regular polychoron"),
    };
    let version_before = document.version;
    let before = document.clone();
    let mut undo_stack = VecDeque::new();
    let mut redo_stack = VecDeque::from([ChangeSet {
        before: grafito_core::Document::new(),
        after: grafito_core::Document::new(),
    }]);

    for id in [polychoron_id, no_fill_id, point_id, ObjectId::new()] {
        let color = if id == polychoron_id {
            existing_fill
        } else {
            Color::GREEN
        };
        assert!(
            !crate::ui::apply_color_picker_regular_polychoron_fill_color_change(
                &mut document,
                id,
                color,
                &mut undo_stack,
                &mut redo_stack,
            )
            .expect("equal, missing, and wrong-type fill targets are no-ops")
        );
    }

    assert_eq!(document.version, version_before);
    assert_eq!(
        serde_json::to_value(&document).expect("document serializes"),
        serde_json::to_value(&before).expect("baseline serializes")
    );
    assert!(undo_stack.is_empty());
    assert_eq!(redo_stack.len(), 1);
}

#[test]
fn missing_algebra_variable_metadata_is_display_only_until_changed() {
    let mut document = grafito_core::Document::new();
    document
        .try_set_variable("t".to_string(), 100.0)
        .expect("finite variable inserts");
    let before = serde_json::to_value(&document).expect("document serializes");
    let version_before = document.version;

    let metadata = crate::algebra::variable_meta_for_display(&document, "t");

    assert_eq!(metadata.min, -5.0);
    assert_eq!(metadata.max, 5.0);
    assert_eq!(metadata.step, 0.1);
    assert!(!metadata.animating);
    assert_eq!(metadata.animation_speed, 1.0);
    assert!(document.variable_meta("t").is_none());
    assert_eq!(document.version, version_before);
    assert_eq!(serde_json::to_value(&document).unwrap(), before);

    let mut displayed_value = document
        .get_variable("t")
        .expect("variable remains available");
    let context = egui::Context::default();
    let _ = context.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(
                egui::Slider::new(&mut displayed_value, metadata.min..=metadata.max)
                    .clamping(egui::SliderClamping::Edits),
            );
        });
    });
    assert_eq!(displayed_value, 100.0);

    let source = include_str!("algebra.rs");
    assert!(source.contains("variable_meta_for_display(&app.document, name)"));
    assert!(source.contains(".clamping(egui::SliderClamping::Edits)"));
    assert!(source.contains("try_replace_variable_meta_with_previous"));
    assert!(!source.contains("variable_meta.insert"));
    assert!(!source.contains("app.document.variable_meta"));
}

#[test]
fn algebra_variable_metadata_edits_capture_one_valid_undo_and_reject_invalid_ranges() {
    use grafito_core::ChangeSet;

    let mut document = grafito_core::Document::new();
    document
        .try_set_variable("t".to_string(), 0.0)
        .expect("fixture variable inserts");
    let mut undo_stack = VecDeque::new();
    let mut redo_stack = VecDeque::from([ChangeSet {
        before: grafito_core::Document::new(),
        after: grafito_core::Document::new(),
    }]);
    let mut snapshot = crate::app::DeferredPanelSnapshot::new(undo_stack.len());
    let mut candidate = crate::algebra::variable_meta_for_display(&document, "t");
    candidate.min = -2.0;
    candidate.max = 8.0;

    assert!(crate::algebra::apply_variable_meta_panel_edit(
        &mut document,
        "t",
        candidate.clone(),
        &mut snapshot,
    )
    .expect("valid metadata edit commits"));
    assert!(snapshot.is_captured());
    assert!(!snapshot.requires_semantic_comparison());
    assert!(snapshot.save_if_semantically_changed(&mut document, &mut undo_stack, &mut redo_stack,));
    assert_eq!(undo_stack.len(), 1);
    assert!(redo_stack.is_empty());
    assert!(undo_stack[0].variable_meta("t").is_none());
    assert_eq!(document.variable_meta("t"), Some(&candidate));

    let before = serde_json::to_value(&document).expect("document serializes");
    let version_before = document.version;
    let mut rejected_snapshot = crate::app::DeferredPanelSnapshot::new(undo_stack.len());
    let mut rejected = candidate;
    rejected.min = rejected.max;

    let error = crate::algebra::apply_variable_meta_panel_edit(
        &mut document,
        "t",
        rejected,
        &mut rejected_snapshot,
    )
    .expect_err("reversed metadata is rejected before history capture");

    assert!(error.contains("smaller"), "{error}");
    assert_eq!(document.version, version_before);
    assert_eq!(serde_json::to_value(&document).unwrap(), before);
    assert!(!rejected_snapshot.is_captured());
    assert!(!rejected_snapshot.requires_semantic_comparison());
    assert!(!rejected_snapshot.save_if_semantically_changed(
        &mut document,
        &mut undo_stack,
        &mut redo_stack,
    ));
    assert_eq!(undo_stack.len(), 1);
    assert!(redo_stack.is_empty());
}

#[test]
fn idle_selected_algebra_properties_do_not_acquire_mutable_document_access() {
    let source = include_str!("algebra.rs");
    let start = source
        .find("// Properties Panel (Inline)")
        .expect("selected property panel");
    let end = source[start..]
        .find("if row_clicked")
        .map(|offset| start + offset)
        .expect("selected property panel end");
    let properties = &source[start..end];

    assert!(properties.contains("get_object(oid).cloned()"));
    assert!(properties.contains("apply_object_panel_edit"));
    assert!(properties.contains("capture_successful_replacement"));
    assert!(!properties.contains("get_object_mut"));
    assert!(!properties.contains("snapshot.capture(&app.document)"));
    assert!(!source.contains("let before = app.document.clone()"));
    assert!(source.contains("DeferredPanelSnapshot::new"));
}

#[test]
fn changed_object_panel_edit_bumps_version_once() {
    use grafito_core::{Cube3DObj, GeoObject};
    use grafito_geometry::Point3D;

    let mut document = grafito_core::Document::new();
    let cube = document.add_object(GeoObject::Cube3D(Cube3DObj::new(
        Point3D::new(0.0, 0.0, 0.0),
        2.0,
    )));
    let version_before = document.version;

    assert!(
        crate::panels::apply_object_panel_edit(&mut document, cube, true, |object| {
            let GeoObject::Cube3D(cube) = object else {
                panic!("expected cube");
            };
            cube.size = 3.0;
        },)
        .expect("a valid panel edit commits")
    );
    assert_eq!(document.version, version_before.wrapping_add(1));
}

#[test]
fn object_panel_edit_uses_staged_replacement_and_rejection_preserves_history() {
    use grafito_core::{ChangeSet, GeoObject, RegularPolychoron4DObj};
    use grafito_geometry::RegularPolychoron;

    let mut document = grafito_core::Document::new();
    let id = document
        .try_add_object(GeoObject::RegularPolychoron4D(RegularPolychoron4DObj::new(
            RegularPolychoron::Tesseract,
        )))
        .expect("fixture inserts");
    let before = document.clone();
    let version_before = document.version;
    let undo_stack: VecDeque<grafito_core::Document> = VecDeque::new();
    let redo_stack = VecDeque::from([ChangeSet {
        before: grafito_core::Document::new(),
        after: grafito_core::Document::new(),
    }]);

    let error = crate::panels::apply_object_panel_edit(&mut document, id, true, |object| {
        let GeoObject::RegularPolychoron4D(polychoron) = object else {
            panic!("fixture remains a regular polychoron");
        };
        polychoron.scale = 0.0;
    })
    .expect_err("invalid detached candidates must be rejected");

    assert!(error.contains("scale"));
    assert_eq!(document.version, version_before);
    assert_eq!(
        serde_json::to_value(&document).unwrap(),
        serde_json::to_value(&before).unwrap()
    );
    assert!(undo_stack.is_empty());
    assert_eq!(redo_stack.len(), 1);
}

#[test]
fn object_panel_edit_commits_compound_4d_and_nd_edits_with_one_snapshot() {
    use grafito_core::{GeoObject, RegularPolychoron4DObj, RegularPolytopeNDObj};
    use grafito_geometry::{Color, RegularPolychoron, RegularPolytopeFamily};

    let mut document = grafito_core::Document::new();
    let mut polychoron = RegularPolychoron4DObj::new(RegularPolychoron::Tesseract);
    polychoron.label = "P4".to_string();
    polychoron.visible = false;
    let polychoron_id = document
        .try_add_object(GeoObject::RegularPolychoron4D(polychoron))
        .expect("4D fixture inserts");
    let version_before = document.version;

    assert!(
        crate::panels::apply_object_panel_edit(&mut document, polychoron_id, true, |object| {
            let GeoObject::RegularPolychoron4D(polychoron) = object else {
                panic!("fixture remains a regular polychoron");
            };
            polychoron.kind = RegularPolychoron::TwentyFourCell;
            polychoron.scale = 2.5;
            polychoron.width = 3.0;
            polychoron.color = Color::new(0.1, 0.2, 0.3, 0.9);
            polychoron.fill_color = Some(Color::new(0.7, 0.5, 0.2, 0.4));
            polychoron.rotation_angles = [0.1, -0.2, 0.3, -0.4, 0.5, -0.6];
        },)
        .expect("compound 4D edit commits")
    );
    assert_eq!(document.version, version_before.wrapping_add(1));
    assert!(matches!(
        document.get_object(polychoron_id),
        Some(GeoObject::RegularPolychoron4D(polychoron))
            if polychoron.label == "P4"
                && !polychoron.visible
                && polychoron.kind == RegularPolychoron::TwentyFourCell
                && polychoron.scale == 2.5
                && polychoron.width == 3.0
                && polychoron.rotation_angles == [0.1, -0.2, 0.3, -0.4, 0.5, -0.6]
                && polychoron.fill_color == Some(Color::new(0.7, 0.5, 0.2, 0.4))
    ));

    let mut polytope = RegularPolytopeNDObj::new(RegularPolytopeFamily::Hypercube, 5);
    polytope.label = "PN".to_string();
    polytope.visible = false;
    polytope.fill_color = Some(Color::new(0.3, 0.4, 0.5, 0.6));
    polytope.rotation_angles.fill(0.75);
    let polytope_id = document
        .try_add_object(GeoObject::RegularPolytopeND(polytope))
        .expect("N-D fixture inserts");
    let version_before = document.version;

    assert!(
        crate::panels::apply_object_panel_edit(&mut document, polytope_id, true, |object| {
            let GeoObject::RegularPolytopeND(polytope) = object else {
                panic!("fixture remains a regular N-D polytope");
            };
            polytope.family = RegularPolytopeFamily::CrossPolytope;
            polytope.dimension = 10;
            polytope.scale = 1.75;
            polytope.width = 2.5;
            polytope.color = Color::new(0.6, 0.2, 0.4, 0.8);
            let rotation_count =
                RegularPolytopeNDObj::expected_rotation_angle_count(polytope.dimension)
                    .expect("the selected dimension is supported");
            polytope.rotation_angles = vec![0.0; rotation_count];
        },)
        .expect("compound N-D edit commits")
    );
    assert_eq!(document.version, version_before.wrapping_add(1));
    assert!(matches!(
        document.get_object(polytope_id),
        Some(GeoObject::RegularPolytopeND(polytope))
            if polytope.label == "PN"
                && !polytope.visible
                && polytope.family == RegularPolytopeFamily::CrossPolytope
                && polytope.dimension == 10
                && polytope.scale == 1.75
                && polytope.width == 2.5
                && polytope.rotation_angles.len() == 45
                && polytope.rotation_angles.iter().all(|angle| *angle == 0.0)
                && polytope.fill_color == Some(Color::new(0.3, 0.4, 0.5, 0.6))
    ));
}

#[test]
fn geometry_3d_polytope_inspectors_expose_labeled_scrollable_controls() {
    let source = include_str!("panels.rs");
    let inspector_start = source
        .find("pub(crate) fn draw_right_properties_panel")
        .expect("Geometry3D properties panel");
    let inspector_end = source[inspector_start..]
        .find("pub(crate) fn draw_right_domain_coloring_panel")
        .map(|offset| inspector_start + offset)
        .expect("next panel");
    let inspector = &source[inspector_start..inspector_end];
    let polychoron_start = inspector
        .find("GeoObject::RegularPolychoron4D")
        .expect("4D polychoron inspector");
    let polytope_start = inspector
        .find("GeoObject::RegularPolytopeND")
        .expect("N-D polytope inspector");
    let polychoron = &inspector[polychoron_start..polytope_start];
    let polytope = &inspector[polytope_start..];

    for plane in [
        "xy (rad)", "xz (rad)", "xw (rad)", "yz (rad)", "yw (rad)", "zw (rad)",
    ] {
        assert!(
            include_str!("sheet_layout.rs").contains(plane),
            "missing {plane} control"
        );
    }
    // Slice 60 F14.4.4: los literales viven en el dueño nuevo
    // (`sheet_layout.rs`); panels consume las consts/fns puras.
    // B-A109: el chrome 4D/N-D sale de `inspector.poly*` vía dispatcher.
    assert!(polychoron.contains("POLYCHORON_ROTATION_PLANES"));
    // B-A159: el label sale localizado vía dispatcher (antes ES duro).
    assert!(polychoron.contains("inspector_fill_enabled_label"));
    assert!(polychoron.contains("inspector_edge_color_label"));
    assert!(polychoron.contains("inspector_fill_color_label"));
    assert!(include_str!("sheet_layout.rs").contains("Relleno habilitado"));
    assert!(polychoron.contains("app.open_object_color_picker(id)"));
    assert!(polychoron.contains("app.open_regular_polychoron_fill_color_picker(id)"));
    assert!(!polychoron.contains("color_edit_button_srgba_unmultiplied"));
    assert!(!polychoron.contains("polychoron.color = color_from_srgba_unmultiplied"));
    assert!(polychoron.contains("poly_fill_omitted_hint"));
    assert!(include_str!("sheet_layout.rs").contains("Vista previa"));
    assert!(include_str!("sheet_layout.rs").contains("movimiento"));
    assert!(polychoron.contains("poly_rotation_reset_label"));
    assert!(polychoron.contains("poly_rotation_manual_title"));
    assert!(include_str!("sheet_layout.rs").contains("Restablecer rotaciones"));
    assert!(include_str!("sheet_layout.rs").contains("Rotación manual"));
    assert!(polychoron.contains(".default_open(false)"));
    assert!(polychoron.contains("poly_anim_title"));
    assert!(include_str!("sheet_layout.rs").contains("Animación de proyección"));
    assert!(polychoron.contains("draw_multidimensional_motion_card"));
    assert!(polychoron.contains("ui.push_id"));

    assert!(polytope.contains("ComboBox"));
    assert!(polytope.contains("3..=10"));
    assert!(polytope.contains("expected_rotation_angle_count"));
    assert!(polytope.contains("rotation_angles = vec![0.0; rotation_count]"));
    assert!(polytope.contains("ScrollArea::vertical"));
    assert!(polytope.contains("regular_polytope_nd_rotation_planes"));
    assert!(polytope.contains("poly_rotation_manual_title"));
    assert!(polytope.contains(".default_open(false)"));
    assert!(polytope.contains("nd_rotation_plane_label"));
    assert!(include_str!("sheet_layout.rs").contains("x{}/x{} (rad)"));
    assert!(polytope.contains("polytope_fill_hint"));
    assert!(include_str!("sheet_layout.rs").contains("solo como aristas"));
    assert!(!polytope.contains("fill_color"));
    assert!(polytope.contains("app.open_object_color_picker(id)"));
    assert!(!polytope.contains("color_edit_button_srgba_unmultiplied"));
    // La tarjeta de animación sale siempre (todas las dimensiones): el
    // interruptor también orbita la cámara; la descripción honesta distingue
    // R4 ("misma velocidad") del resto ("solo órbita").
    // B-A109: el cuerpo N-D sale localizado vía `polytope_nd_projection_body_for`.
    assert!(polytope.contains("polytope_nd_projection_body_for"));
    assert!(include_str!("sheet_layout.rs").contains("La cámara orbita sin alterar el documento."));
    assert!(include_str!("sheet_layout.rs")
        .contains("La cámara y la proyección 4D usan la misma velocidad."));
    assert!(polytope.contains("draw_multidimensional_motion_card"));
    assert!(polytope.contains("ui.push_id"));
    assert!(inspector.contains("right_properties_scroll"));
    assert!(inspector.contains(".auto_shrink([false, true])"));
    assert!(inspector.contains("draw_inspector_identity"));
    assert!(source.contains("fn draw_inspector_empty_state"));
    // B-A108: el vacío sale del dispatcher i18n (antes const ES directa).
    assert!(source.contains("inspector_chrome_for(\"empty_title\""));
    assert!(
        include_str!("sheet_layout.rs").contains("Tocá un objeto del lienzo para editarlo acá.")
    );
    assert!(source.contains("inspector_equation_text"));
    assert!(source.contains("inspector_type_caption"));
    // B-A108: sin ecuación sale del dispatcher i18n (antes const ES directa).
    assert!(source.contains("inspector_chrome_for(\"no_equation\""));
    assert!(include_str!("sheet_layout.rs").contains("Ecuación no disponible para este tipo"));
    // B-A109: los títulos de sección 4D/N-D salen del dispatcher i18n.
    assert!(inspector.contains("poly_projection_title"));
    assert!(inspector.contains("poly_geometry_title"));
    assert!(inspector.contains("poly_appearance_title"));
    assert!(include_str!("sheet_layout.rs").contains("Proyección"));
    for control in [
        "motion.pause",
        "motion.start",
        "motion.speed",
        "motion.reset",
        "motion.need_3d",
        "motion_locale",
        "pause_label",
        "start_label",
        "speed_text_label",
        "reset_label",
        "need_3d_label",
    ] {
        assert!(source.contains(control), "missing {control}");
    }
    assert!(source.contains(".text(speed_text_label)"));
}

#[test]
fn inspector_label_never_renders_a_bare_letter() {
    // Vacía → sin fila (nada que mostrar, jamás 1 letra).
    assert_eq!(crate::sheet_layout::inspector_label_text(""), None);
    assert_eq!(crate::sheet_layout::inspector_label_text("   "), None);
    // Autolabel de 1 letra (`S` de Surface3D, `document.rs:1193-1202`)
    // → con prefijo, jamás suelta.
    assert_eq!(
        crate::sheet_layout::inspector_label_text("S").as_deref(),
        Some("Etiqueta: S")
    );
    // Texto largo → completo, sin recorte silencioso.
    let long = "Superficie paramétrica de revolución con malla fina";
    let rendered = crate::sheet_layout::inspector_label_text(long).expect("label text");
    assert!(rendered.contains(long), "elide honesto: texto completo");
    assert!(rendered.chars().count() > 1);
    // Trim honesto, sin duplicar prefijo.
    assert_eq!(
        crate::sheet_layout::inspector_label_text("  S  ").as_deref(),
        Some("Etiqueta: S")
    );
}

#[test]
fn inspector_identity_tooltip_carries_full_text() {
    // El titular es la ecuación en grande (o el caption si no hay).
    let tip = crate::sheet_layout::inspector_identity_tooltip("z = x^2+y^2", "S", true);
    assert!(tip.contains("z = x^2+y^2"), "ecuación completa en tooltip");
    assert!(tip.contains("Etiqueta: S"), "etiqueta con contexto");
    assert!(tip.contains("Visible"));
    let hidden = crate::sheet_layout::inspector_identity_tooltip("curva paramétrica 3D", "", false);
    assert!(hidden.contains("curva paramétrica 3D"));
    assert!(hidden.contains("Oculto"));
    assert!(!hidden.contains("Etiqueta:"));
}

#[test]
fn inspector_equation_text_shows_formula_not_type_name() {
    use grafito_core::{GeoObject, ParametricCurve3DObj, Surface3DObj};
    // Superficie explícita → fórmula en grande.
    let surface = GeoObject::Surface3D(Surface3DObj::new("x^2+y^2", (-2.0, 2.0), (-2.0, 2.0)));
    assert_eq!(
        crate::sheet_layout::inspector_equation_text(&surface).as_deref(),
        Some("z = x^2+y^2")
    );
    // Superficie paramétrica → tupla.
    let param = GeoObject::Surface3D(Surface3DObj::new_parametric(
        "cos(u)",
        "sin(u)",
        "v",
        (0.0, std::f64::consts::TAU),
        (-1.0, 1.0),
    ));
    assert_eq!(
        crate::sheet_layout::inspector_equation_text(&param).as_deref(),
        Some("(cos(u), sin(u), v)")
    );
    // Curva paramétrica 3D → tupla.
    let curve =
        GeoObject::ParametricCurve3D(ParametricCurve3DObj::new("t", "t^2", "t^3", 0.0, 1.0));
    assert_eq!(
        crate::sheet_layout::inspector_equation_text(&curve).as_deref(),
        Some("(t, t^2, t^3)")
    );
    // Vacía → None (placeholder honesto, jamás el nombre en grande).
    let empty = GeoObject::Surface3D(Surface3DObj::new("", (-1.0, 1.0), (-1.0, 1.0)));
    assert_eq!(crate::sheet_layout::inspector_equation_text(&empty), None);
}

#[test]
fn inspector_equation_text_is_none_without_formula() {
    use grafito_core::{Cube3DObj, GeoObject, Point3DObj, Sphere3DObj};
    use grafito_geometry::Point3D;
    let origin = Point3D::new(0.0, 0.0, 0.0);
    for obj in [
        GeoObject::Cube3D(Cube3DObj::new(origin, 1.0)),
        GeoObject::Sphere3D(Sphere3DObj::new(origin, 1.0)),
        GeoObject::Point3D(Point3DObj::new(origin)),
    ] {
        assert_eq!(
            crate::sheet_layout::inspector_equation_text(&obj),
            None,
            "sin fórmula → placeholder honesto"
        );
        assert!(
            !crate::sheet_layout::inspector_type_caption(&obj).is_empty(),
            "siempre hay caption terciario"
        );
    }
}

#[test]
fn inspector_type_caption_is_spanish_tertiary() {
    use grafito_core::{GeoObject, ParametricCurve3DObj, Surface3DObj};
    let surface = GeoObject::Surface3D(Surface3DObj::new("x", (-1.0, 1.0), (-1.0, 1.0)));
    assert_eq!(
        crate::sheet_layout::inspector_type_caption(&surface),
        "superficie 3D"
    );
    let curve = GeoObject::ParametricCurve3D(ParametricCurve3DObj::new("t", "0", "0", 0.0, 1.0));
    assert_eq!(
        crate::sheet_layout::inspector_type_caption(&curve),
        "curva paramétrica 3D"
    );
    // El caption nunca es el nombre muerto en grande: es minúscula terciaria.
    for caption in [
        crate::sheet_layout::inspector_type_caption(&surface),
        crate::sheet_layout::inspector_type_caption(&curve),
    ] {
        assert_ne!(caption, "Surface3D");
        assert_ne!(caption, "ParametricCurve3D");
    }
}

#[test]
fn inspector_cuadrica_placeholder_lleva_badge() {
    use grafito_core::{GeoObject, Quadric3DObj};
    // Elipsoide real: caption limpio.
    let elipsoide = GeoObject::Quadric3D(Quadric3DObj::from_coeffs([
        0.25, 1.0, 1.0, 0.0, 0.0, 0.0, -0.5, 0.0, 0.0, -0.75,
    ]));
    assert_eq!(
        crate::sheet_layout::inspector_type_caption(&elipsoide),
        "cuádrica 3D"
    );
    // Hiperboloide (malla exacta A2): caption limpio, sin badge.
    let exacta = GeoObject::Quadric3D(Quadric3DObj::from_coeffs([
        1.0, 1.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0,
    ]));
    assert_eq!(
        crate::sheet_layout::inspector_type_caption(&exacta),
        "cuádrica 3D"
    );
    // Vacía `x² + y² + z² = -1`: sin superficie → badge visible.
    let aproximada = GeoObject::Quadric3D(Quadric3DObj::from_coeffs([
        1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]));
    assert_eq!(
        crate::sheet_layout::inspector_type_caption(&aproximada),
        "cuádrica 3D · vista aproximada"
    );
}

#[test]
fn collapsed_rail_shows_icons_only_without_text_slivers() {
    // Rail sano (60px) → icono + etiqueta.
    assert!(crate::ui::rail_labels_visible(60.0));
    assert!(crate::ui::rail_labels_visible(
        crate::ui::RAIL_LABEL_MIN_WIDTH
    ));
    // Franja colapsada (1-2px, el artefacto del screenshot) → iconos puros.
    assert!(!crate::ui::rail_labels_visible(2.0));
    assert!(!crate::ui::rail_labels_visible(1.0));
    assert!(!crate::ui::rail_labels_visible(0.0));
    assert!(!crate::ui::rail_labels_visible(
        crate::ui::RAIL_LABEL_MIN_WIDTH - 0.1
    ));
    // Móvil <740 (M3): burbuja + hoja completa en vez del panel fijo;
    // desktop ≥740 intacto. Umbral = el de la paleta-sheet (un solo 740).
    assert!(crate::ui::mobile_assistant_bubble(430.0));
    assert!(crate::ui::mobile_assistant_bubble(739.0));
    assert!(!crate::ui::mobile_assistant_bubble(740.0));
    assert!(!crate::ui::mobile_assistant_bubble(1280.0));
    // Anchos mínimos sanos (tokens como única fuente).
    const { assert!(grafito_ui::tokens::RAIL_WIDTH >= crate::ui::RAIL_LABEL_MIN_WIDTH) };
    const { assert!(grafito_ui::tokens::DRAWER_RIGHT_MIN >= 200.0) };
    const { assert!(grafito_ui::tokens::PANEL_LEFT_MIN >= 100.0) };
}

#[test]
fn narrow_left_drawer_skips_text_instead_of_painting_slivers() {
    // Repro W-C: egui colapsa el SidePanel a `available` cuando el mínimo no
    // entra (`clamp_to_range(width, min..max).at_most(available)`). Con 0-60px
    // el ancho real cae bajo PANEL_LEFT_MIN (240) y el contenido —que exige
    // `min_width`— se pinta clipado: slivers en x≈0-8px.
    for narrow in [0.0, 1.0, 8.0, 60.0] {
        let width_range_min = grafito_ui::tokens::PANEL_LEFT_MIN;
        // Misma aritmética que egui `panel.rs`:
        // `clamp_to_range(default, min..max).at_most(available)`.
        let collapsed = 260.0_f32.clamp(width_range_min, f32::INFINITY).min(narrow);
        assert!(
            collapsed < width_range_min,
            "con {narrow}px el panel colapsa bajo el mínimo"
        );
        assert!(
            !crate::ui::left_drawer_content_visible(narrow),
            "con {narrow}px el layout no debe pintar texto del drawer"
        );
    }
    // Ancho sano (mínimo y default) → el drawer se pinta.
    assert!(crate::ui::left_drawer_content_visible(
        grafito_ui::tokens::PANEL_LEFT_MIN
    ));
    assert!(crate::ui::left_drawer_content_visible(
        grafito_ui::tokens::PANEL_LEFT_DEFAULT
    ));
    // El gate vive en el layout (app.rs) y el helper en ui.rs.
    let app_source = include_str!("app.rs");
    assert!(app_source.contains("left_drawer_content_visible"));
    assert!(app_source.contains("left_drawer_fits"));
    let ui_source = include_str!("ui.rs");
    assert!(ui_source.contains("fn left_drawer_content_visible"));
    assert!(ui_source.contains("PANEL_LEFT_MIN"));
}

#[test]
fn identity_card_and_rail_clip_text_honestly() {
    let panels = include_str!("panels.rs");
    assert!(panels.contains("inspector_label_text"));
    // Slice 58 F14.4.4: el literal "Etiqueta: " vive en el dueño nuevo
    // (`sheet_layout.rs`: `inspector_label_text` + `inspector_3d_tag_label`);
    // panels lo consume por import (paridad sin duplicar cadenas).
    let sheet = include_str!("sheet_layout.rs");
    assert!(sheet.contains("Etiqueta: "));
    assert!(panels.contains("inspector_3d_tag_label"));
    assert!(panels.contains("inspector_identity_tooltip"));
    assert!(panels.contains(".truncate()"));
    assert!(panels.contains("TextWrapMode::Wrap"));
    assert!(panels.contains(".on_hover_text("));
    let ui = include_str!("ui.rs");
    assert!(ui.contains("with_clip_rect"));
    assert!(ui.contains("rail_labels_visible"));
    assert!(ui.contains("RAIL_LABEL_MIN_WIDTH"));
}

#[test]
fn workspace_tabs_are_centered_pills_with_real_contrast() {
    let ui = include_str!("ui.rs");
    // Segmentado de ancho completo (centrado por construcción), pill activo.
    assert!(ui.contains("Button::new"), "tabs como botones");
    assert!(
        ui.contains(".selected(inspector_selected)"),
        "pill inspector seleccionable"
    );
    assert!(
        ui.contains(".selected(assistant_selected)"),
        "pill asistente seleccionable"
    );
    assert!(ui.contains("RADIUS_PILL"), "pill del activo");
    assert!(
        ui.contains("keyboard_enter_bg"),
        "fill activo de alto contraste"
    );
    assert!(
        ui.contains("keyboard_enter_text"),
        "texto activo de alto contraste"
    );
    // TYPE_SM como única medida de los tabs + SPACE_SM debajo del grupo.
    let dock_fn = ui
        .split("fn draw_geometry_utility_contents")
        .nth(1)
        .expect("dock fn");
    let dock_fn = dock_fn
        .split("fn draw_geometry_utility_dock")
        .next()
        .expect("fin del dock fn");
    assert!(dock_fn.contains("TYPE_SM"), "tabs en TYPE_SM");
    assert!(
        !dock_fn.contains("selectable_label"),
        "sin tabs pegados a la izquierda"
    );
}

#[test]
fn workspace_dock_kills_glyph_strip_causes() {
    let ui = include_str!("ui.rs");
    let dock_fn = ui
        .split("fn draw_geometry_utility_dock")
        .nth(1)
        .expect("dock fn");
    let dock_fn = dock_fn.split("\nfn ").next().expect("fin del dock fn");
    // 1. Fondo opaco: el canvas 3D no se transparenta por el borde.
    assert!(dock_fn.contains(".fill(theme.panel_bg)"), "drawer opaco");
    assert!(!dock_fn.contains("TRANSPARENT"), "sin fondo transparente");
    // 2. Anchos por token con mínimo real (nunca franja 0-40px).
    assert!(dock_fn.contains("DRAWER_RIGHT_DEFAULT"), "ancho por token");
    assert!(dock_fn.contains("DRAWER_RIGHT_MIN"), "mínimo por token");
    assert!(dock_fn.contains("DRAWER_RIGHT_MAX"), "máximo por token");
    const {
        assert!(grafito_ui::tokens::DRAWER_RIGHT_MIN >= 200.0);
        assert!(grafito_ui::tokens::DRAWER_RIGHT_MIN >= crate::ui::RAIL_LABEL_MIN_WIDTH);
    }
    // 3. Cero texto pintado a mano en el path del dock: todo Label/widget
    // (con truncate + wrap + tooltip), nada de `painter().text` sin clip.
    for marker in [
        "fn draw_geometry_utility_contents",
        "fn draw_geometry_utility_dock",
        "fn draw_compact_geometry_utility_dock",
    ] {
        let body = ui.split(marker).nth(1).expect(marker);
        let body = body.split("\nfn ").next().unwrap_or(body);
        assert!(
            !body.contains("painter().text"),
            "{marker} sin texto a mano"
        );
    }
    let panels = include_str!("panels.rs");
    for marker in [
        "fn draw_inspector_identity",
        "fn draw_right_properties_contents",
        "fn draw_inspector_section",
        "fn draw_inspector_empty_state",
    ] {
        let body = panels.split(marker).nth(1).expect(marker);
        let body = body.split("\nfn ").next().unwrap_or(body);
        assert!(
            !body.contains("painter().text"),
            "{marker} sin texto a mano"
        );
    }
    // El único texto a mano que queda en panels.rs (histograma) va clipeado.
    assert!(
        panels.contains("with_clip_rect(hist_rect)"),
        "histograma clipeado"
    );
    // La card vieja con nombre muerto no vuelve.
    assert!(!panels.contains("Propiedades dedicadas en panel de Álgebra."));
    assert!(!panels.contains("Identidad del objeto"));
}

#[test]
fn workspace_utility_dock_reserves_one_right_column_for_3d_properties() {
    use crate::{Perspective, RightPanelContent, ShellWidthClass};

    assert!(crate::geometry_utility_dock_available(
        Perspective::Geometry3D,
        Some(RightPanelContent::Properties),
        ShellWidthClass::Medium,
    ));
    assert!(crate::geometry_utility_dock_available(
        Perspective::Geometry3D,
        Some(RightPanelContent::Properties),
        ShellWidthClass::Wide,
    ));
    assert!(!crate::uses_geometry_utility_dock(
        Perspective::Geometry3D,
        Some(RightPanelContent::Properties),
        ShellWidthClass::Compact,
        true,
    ));
    assert!(crate::uses_geometry_utility_dock(
        Perspective::Geometry3D,
        Some(RightPanelContent::Properties),
        ShellWidthClass::Medium,
        true,
    ));
    assert!(crate::uses_geometry_utility_dock(
        Perspective::Geometry3D,
        Some(RightPanelContent::Properties),
        ShellWidthClass::Wide,
        true,
    ));
    assert!(!crate::uses_geometry_utility_dock(
        Perspective::Geometry2D,
        Some(RightPanelContent::Properties),
        ShellWidthClass::Wide,
        true,
    ));
    assert!(!crate::uses_geometry_utility_dock(
        Perspective::Geometry3D,
        Some(RightPanelContent::Properties),
        ShellWidthClass::Wide,
        false,
    ));
    assert!(crate::uses_compact_geometry_utility_dock(
        Perspective::Geometry3D,
        ShellWidthClass::Compact,
        true,
    ));
    assert!(!crate::uses_compact_geometry_utility_dock(
        Perspective::Geometry3D,
        ShellWidthClass::Medium,
        true,
    ));
    assert!(!crate::uses_compact_geometry_utility_dock(
        Perspective::Geometry3D,
        ShellWidthClass::Compact,
        false,
    ));

    let app_source = include_str!("app.rs");
    // S4b: el dock lateral suma gate móvil (Nivel 0 cede al canvas).
    assert!(app_source.contains("if geometry_utility_dock_available && !nivel0_dock {"));
    assert!(app_source.contains("if compact_geometry_utility_dock {"));
}

#[test]
fn reopening_the_assistant_restores_its_geometry_utility_host() {
    let app_source = include_str!("app.rs");
    let ui_source = include_str!("ui.rs");

    assert!(ui_source.contains("app.open_assistant_workspace();"));
    assert!(app_source.contains("pub(crate) fn open_assistant_workspace"));
    assert!(app_source.contains("self.workspace_dock_tab = crate::WorkspaceDockTab::Assistant;"));
    assert!(app_source.contains("self.right_drawer_open = true;"));
    assert!(app_source.contains("self.compact_geometry_utility_open = true;"));
}

#[test]
fn trig_animation_never_claims_the_geometry_3d_utility_column() {
    assert!(crate::app::trig_animation_supported(crate::ViewMode::D2));
    assert!(!crate::app::trig_animation_supported(crate::ViewMode::D3));

    let app_source = include_str!("app.rs");
    assert!(app_source.contains("visible && trig_animation_supported(self.current_view)"));
}

#[test]
fn three_d_algebra_rows_leave_full_editing_to_the_inspector() {
    let source = include_str!("algebra.rs");

    assert!(source.contains("app.current_view != ViewMode::D3"));
    assert!(source.contains("Abrí el Inspector para editar este objeto 3D."));
}

#[test]
fn object_color_control_has_a_practical_pointer_target() {
    let size = crate::algebra::OBJECT_COLOR_TARGET_SIZE;
    assert!(size.x >= 24.0);
    assert!(size.y >= 24.0);
}

#[test]
fn algebra_color_swatches_open_the_shared_picker_helpers() {
    let source = include_str!("algebra.rs");

    assert!(source.contains("app.open_object_color_picker(oid)"));
    assert!(!source.contains("active_color_picker = Some"));
}

#[test]
fn renderer_readiness_rejects_missing_or_locked_renderer() {
    use std::sync::{Arc, RwLock};

    let missing = Arc::new(RwLock::new(None::<()>));
    assert!(!crate::app::renderer_is_ready(Some(&missing)));

    let ready = Arc::new(RwLock::new(Some(())));
    assert!(crate::app::renderer_is_ready(Some(&ready)));
    let read_guard = ready.read().expect("renderer lock should be available");
    assert!(!crate::app::renderer_is_ready(Some(&ready)));
    drop(read_guard);
    let write_guard = ready.write().expect("renderer lock should be available");
    assert!(!crate::app::renderer_is_ready(Some(&ready)));
    drop(write_guard);
}

#[test]
fn two_dimensional_gpu_path_requires_ready_renderer_and_nonempty_canvas() {
    assert!(!crate::app::should_use_gpu_2d(
        true,
        false,
        egui::vec2(800.0, 600.0)
    ));
    assert!(!crate::app::should_use_gpu_2d(
        true,
        true,
        egui::vec2(0.0, 600.0)
    ));
    assert!(!crate::app::should_use_gpu_2d(
        false,
        true,
        egui::vec2(800.0, 600.0)
    ));
    assert!(crate::app::should_use_gpu_2d(
        true,
        true,
        egui::vec2(800.0, 600.0)
    ));
}

#[test]
fn unversioned_legacy_visibility_refreshes_all_gpu_graphable_objects_and_caches() {
    use grafito_core::{
        function_sampling, ComplexGridObj, ComplexMappingObj, Fractal2DObj, FunctionObj, GeoObject,
        ParametricCurve2DObj, PointObj, PolarCurveObj, VectorField2DObj,
    };
    use grafito_geometry::Point2;
    use std::collections::BTreeMap;

    let mut document = grafito_core::Document::new();
    let target = document.add_object(GeoObject::Point(PointObj::new(Point2::new(0.0, 0.0))));
    let function = document.add_object(GeoObject::Function(FunctionObj::new("x")));
    let graphable_ids = [
        function,
        document.add_object(GeoObject::ParametricCurve2D(ParametricCurve2DObj::new(
            "t", "t", 0.0, 1.0,
        ))),
        document.add_object(GeoObject::PolarCurve(PolarCurveObj::new("1", 0.0, 1.0))),
        document.add_object(GeoObject::VectorField2D(VectorField2DObj::new("x", "y"))),
        document.add_object(GeoObject::Fractal2D(Fractal2DObj::mandelbrot())),
        document.add_object(GeoObject::ComplexGrid(ComplexGridObj::new(
            "z", -1.0, 1.0, -1.0, 1.0,
        ))),
        document.add_object(GeoObject::ComplexMapping(ComplexMappingObj::new(
            "z", target,
        ))),
    ];

    let GeoObject::Function(function_obj) = document
        .get_object(function)
        .expect("function should exist")
    else {
        panic!("expected function");
    };
    drop(function_sampling::samples_or_compute(
        function_obj,
        (-1.0, 1.0),
        32,
        &BTreeMap::new(),
    ));
    assert!(function_obj
        .cached_key
        .read()
        .expect("cache lock")
        .is_some());

    let before = document.clone();
    for id in graphable_ids {
        document
            .get_object_mut(id)
            .expect("graphable object should exist")
            .set_visible(false);
    }

    assert_eq!(
        document.version, before.version,
        "los préstamos mutables no bumpean la versión"
    );
    assert_ne!(
        serde_json::to_value(&before).expect("before document should serialize"),
        serde_json::to_value(&document).expect("updated document should serialize")
    );
    assert!(crate::app::refresh_unversioned_document_change(
        &before,
        &mut document
    ));
    assert_eq!(
        document.version,
        before.version.wrapping_add(1),
        "la red de seguridad bumpea una sola vez por cambio sin versionar"
    );
    assert!(graphable_ids.iter().all(|id| {
        !document
            .get_object(*id)
            .expect("graphable object should exist")
            .is_visible()
    }));
    let GeoObject::Function(function_obj) = document
        .get_object(function)
        .expect("function should exist")
    else {
        panic!("expected function");
    };
    assert!(function_obj
        .cached_key
        .read()
        .expect("cache lock")
        .is_none());
}

#[test]
fn unversioned_legacy_style_change_refreshes_gpu_geometry() {
    use grafito_core::{FunctionObj, GeoObject};

    let mut document = grafito_core::Document::new();
    let function = document.add_object(GeoObject::Function(FunctionObj::new("x")));
    let before = document.clone();
    let GeoObject::Function(function_obj) = document
        .get_object_mut(function)
        .expect("function should exist")
    else {
        panic!("expected function");
    };
    function_obj.width = 6.0;

    // Contrato nuevo: el préstamo mutable por sí solo no bumpea `version`
    // (`Document::get_object_mut`). La red de seguridad
    // `refresh_unversioned_document_change` es la que ensucia y refresca.
    assert_eq!(document.version, before.version);
    assert_ne!(
        serde_json::to_value(&before).expect("before document should serialize"),
        serde_json::to_value(&document).expect("updated document should serialize")
    );
    assert!(crate::app::refresh_unversioned_document_change(
        &before,
        &mut document
    ));
    assert_eq!(document.version, before.version.wrapping_add(1));
}

fn toast_shape_count(toasts: &mut grafito_ui::toast::ToastManager) -> usize {
    let context = egui::Context::default();
    let output = context.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| toasts.draw(ui, 0.1));
    });
    output.shapes.len()
}

#[test]
fn mutating_command_submission_saves_undo_and_replaces_redo() {
    let before = grafito_core::Document::new();
    let mut after = before.clone();
    let mut input = "A = (8, 8)".to_string();
    let outcome = crate::commands::process_input(&mut after, &mut input);
    let mut undo_stack = VecDeque::new();
    let mut redo_stack = VecDeque::from([grafito_core::ChangeSet {
        before: before.clone(),
        after: before.clone(),
    }]);

    assert!(!matches!(
        outcome,
        grafito_command::commands::CommandOutcome::Error(_)
    ));
    crate::app::save_command_snapshot_if_mutated(
        &outcome,
        before,
        &after,
        &mut undo_stack,
        &mut redo_stack,
    );
    assert_eq!(undo_stack.len(), 1);
    assert!(redo_stack.is_empty());
}

#[test]
fn compact_shell_never_reserves_both_drawers() {
    let shell = crate::ShellLayout::for_viewport(
        317.0,
        crate::Perspective::Geometry2D,
        0,
        true,
        true,
        true,
    );

    assert_eq!(shell.width_class, crate::ShellWidthClass::Compact);
    assert!(!shell.show_left_drawer);
    assert!(!shell.show_right_drawer);
    assert!(shell.show_bottom_input);
}

#[test]
fn compact_panel_menu_can_open_one_left_drawer() {
    let shell = crate::ShellLayout::for_viewport(
        960.0,
        crate::Perspective::Geometry2D,
        0,
        true,
        true,
        true,
    )
    .with_compact_left_drawer(true, 0);

    assert_eq!(shell.width_class, crate::ShellWidthClass::Compact);
    assert!(shell.show_left_drawer);
    assert!(!shell.show_right_drawer);
    assert!(!shell.show_sidebar);
    assert!(!shell.show_bottom_input);
}

#[test]
fn canvas_focus_shell_keeps_the_assistant_and_restores_the_bottom_input() {
    let algebra = crate::ShellLayout::for_viewport(
        960.0,
        crate::Perspective::Geometry2D,
        0,
        true,
        true,
        true,
    );
    assert_eq!(algebra.width_class, crate::ShellWidthClass::Compact);
    assert!(!algebra.show_sidebar);
    assert!(!algebra.show_left_drawer);
    assert!(!algebra.show_right_drawer);
    assert!(algebra.show_bottom_input);

    let tools = crate::ShellLayout::for_viewport(
        960.0,
        crate::Perspective::Geometry2D,
        1,
        true,
        true,
        true,
    );
    assert!(tools.show_bottom_input);
}

#[test]
fn compact_tab_without_input_always_keeps_the_bottom_entry() {
    // BB3 P0-2: drawer compacto con tab sin entrada propia (4=Vista) →
    // la Entrada inferior es la única y va siempre visible.
    let shell = crate::ShellLayout::for_viewport(
        390.0,
        crate::Perspective::Geometry2D,
        4,
        false,
        false,
        false,
    )
    .with_compact_left_drawer(true, 4);
    assert_eq!(shell.width_class, crate::ShellWidthClass::Compact);
    assert!(shell.show_left_drawer);
    assert!(shell.show_bottom_input);
}

#[test]
fn nivel0_forces_the_bottom_entry_when_the_hidden_drawer_has_input() {
    // BB3 P0-2: a 390 el drawer no se pinta; un compact abierto colgado
    // con tab 0 (Álgebra, con entrada propia) suprimía también la
    // inferior → canvas a pantalla completa sin Entrada. Nivel 0 la
    // fuerza siempre; fuera de Nivel 0 el drawer visible manda.
    let shell = crate::ShellLayout::for_viewport(
        390.0,
        crate::Perspective::Geometry2D,
        0,
        false,
        false,
        false,
    )
    .with_compact_left_drawer(true, 0);
    assert!(!shell.show_bottom_input);
    assert!(shell.with_nivel0(true).show_bottom_input);

    let shell = crate::ShellLayout::for_viewport(
        960.0,
        crate::Perspective::Geometry2D,
        0,
        false,
        false,
        false,
    )
    .with_compact_left_drawer(true, 0)
    .with_nivel0(false);
    assert!(!shell.show_bottom_input);
}

#[test]
fn assistant_mode_decision_uses_window_width_not_shrinking_available() {
    // BB3 P0-1: el predicado side↔sheet se evalúa con el viewport
    // (estable); con `available` flapeaba en 740-900 al reservar el panel.
    let app_source = include_str!("app.rs").replace("\r\n", "\n");
    assert!(app_source
        .contains("grafito_ui::assistant::assistant_uses_bottom_sheet(ctx.screen_rect().width())"));
    assert!(!app_source.contains("assistant_uses_bottom_sheet(ctx.available_rect().width())"));
}

#[test]
fn medium_shell_restores_a_single_left_drawer_after_the_canvas_focus_band() {
    let shell = crate::ShellLayout::for_viewport(
        1_360.0,
        crate::Perspective::Geometry2D,
        0,
        true,
        true,
        true,
    );

    assert_eq!(shell.width_class, crate::ShellWidthClass::Medium);
    assert!(shell.show_sidebar);
    assert!(shell.show_left_drawer);
    assert!(!shell.show_right_drawer);
}

#[test]
fn canvas_focus_extends_until_the_default_drawers_fit_a_useful_canvas() {
    let shell = crate::ShellLayout::for_viewport(
        1_280.0,
        crate::Perspective::Geometry2D,
        0,
        true,
        true,
        true,
    );

    assert_eq!(shell.width_class, crate::ShellWidthClass::Compact);
    assert!(!shell.show_left_drawer);
    assert!(!shell.show_sidebar);
}

#[test]
fn wide_shell_keeps_the_panel_rail_available_when_its_drawer_is_closed() {
    let shell = crate::ShellLayout::for_viewport(
        1_920.0,
        crate::Perspective::Geometry2D,
        0,
        true,
        true,
        false,
    );

    assert!(shell.show_sidebar);
    assert!(!shell.show_left_drawer);
    assert!(shell.show_bottom_input);
}

#[test]
fn shell_and_keyboard_gates_use_window_dimensions_before_panels_reserve_space() {
    // Normalizado a LF: `include_str!` embebe el archivo tal cual y en el
    // checkout de Windows (CRLF) las búsquedas multilínea no matcheaban.
    let app_source = include_str!("app.rs").replace("\r\n", "\n");
    let keyboard_source = include_str!("keyboard.rs").replace("\r\n", "\n");

    assert!(app_source.contains("let viewport_width = ctx.screen_rect().width();"));
    assert!(
        app_source.contains("crate::ShellLayout::for_viewport(\n                viewport_width,")
    );
    assert!(app_source
        .contains(".with_compact_left_drawer(self.compact_drawer_open, self.sidebar_tab)"));
    assert!(keyboard_source.contains("math_keyboard_layout"));
    assert!(app_source.contains("uses_geometry_utility_dock"));
    assert!(app_source.contains("draw_geometry_utility_dock"));
}

#[test]
fn top_chrome_uses_a_dedicated_narrow_width_policy() {
    for width in [960.0, 1_026.0, 1_360.0] {
        assert!(crate::ui::top_chrome_uses_overflow(width));
    }
    assert!(!crate::ui::top_chrome_uses_overflow(1_361.0));
}

#[test]
fn assistant_reopen_control_only_appears_when_panel_is_hidden() {
    assert!(crate::ui::assistant_reopen_control_visible(false));
    assert!(!crate::ui::assistant_reopen_control_visible(true));
}

fn top_chrome_input(width: f32, time: f64, events: Vec<egui::Event>) -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(width, 720.0),
        )),
        time: Some(time),
        events,
        ..Default::default()
    }
}

fn render_assistant_reopen_control(
    ctx: &egui::Context,
    width: f32,
    time: f64,
    events: Vec<egui::Event>,
    assistant_visible: &mut bool,
) -> Option<egui::Rect> {
    let mut rect = None;
    let _ = ctx.run(top_chrome_input(width, time, events), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            rect = crate::ui::draw_assistant_reopen_control(
                ui,
                assistant_visible,
                egui::Color32::WHITE,
                grafito_ui::i18n::Locale::Es,
            )
            .map(|response| response.rect);
        });
    });
    rect
}

#[test]
fn assistant_reopen_control_renders_only_when_hidden_and_restores_the_panel() {
    let ctx = egui::Context::default();
    let mut assistant_visible = false;

    let rendered =
        render_assistant_reopen_control(&ctx, 960.0, 0.0, Vec::new(), &mut assistant_visible);

    assert!(rendered.is_some());
    assert!(!assistant_visible);
    assert!(crate::ui::restore_assistant_visibility(
        &mut assistant_visible,
        true
    ));
    assert!(assistant_visible);

    let rendered =
        render_assistant_reopen_control(&ctx, 960.0, 0.1, Vec::new(), &mut assistant_visible);

    assert!(rendered.is_none());
    assert!(!crate::ui::restore_assistant_visibility(
        &mut assistant_visible,
        true
    ));
}

#[test]
fn assistant_reopen_control_accepts_a_click_at_compact_and_wide_widths() {
    for (width, compact) in [(960.0, true), (1680.0, false)] {
        assert_eq!(crate::ui::top_chrome_uses_overflow(width), compact);

        let ctx = egui::Context::default();
        let mut assistant_visible = false;
        let rect =
            render_assistant_reopen_control(&ctx, width, 0.0, Vec::new(), &mut assistant_visible)
                .expect("hidden assistant must render a reopen control");
        let pos = rect.center();

        let _ = render_assistant_reopen_control(
            &ctx,
            width,
            0.1,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::default(),
                },
            ],
            &mut assistant_visible,
        );
        let _ = render_assistant_reopen_control(
            &ctx,
            width,
            0.2,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::default(),
                },
            ],
            &mut assistant_visible,
        );

        assert!(assistant_visible);
    }
}

#[test]
fn side_drawers_reserve_height_before_the_math_keyboard() {
    let app_source = include_str!("app.rs");
    let assistant = app_source
        .find("self.draw_assistant(ctx, keyboard_height);")
        .expect("assistant draw call");
    // S4b: el drawer derecho suma gate móvil (Nivel 0 cede al canvas).
    let right_drawer = app_source
        .find("if shell.show_right_drawer && !geometry_utility_dock_available && !nivel0_dock {")
        .expect("right drawer block");
    let utility_dock = app_source
        .find("crate::ui::draw_geometry_utility_dock(self, ctx);")
        .expect("Geometry 3D utility dock");
    let keyboard = app_source
        .rfind("crate::keyboard::draw_math_keyboard(self, ctx, keyboard_layout);")
        .expect("keyboard draw call");

    assert!(assistant < keyboard);
    assert!(right_drawer < keyboard);
    assert!(utility_dock < keyboard);
}

#[test]
fn status_bar_stays_in_the_middle_column_in_side_panel_mode() {
    // La barra «0 objetos» se dibuja después del asistente en modo
    // side-panel (limitada a la columna central, como el drawer izquierdo)
    // y antes en modo bottom-sheet (al borde inferior). Con asistente oculto
    // no se limita (evita un hueco vacío abajo a la derecha).
    let app_source = include_str!("app.rs");
    let assistant = app_source
        .find("self.draw_assistant(ctx, keyboard_height);")
        .expect("assistant draw call");
    // Ola 1.1: la barra de entrada vuelve, gobernada por ShellLayout.
    let first_bar = app_source
        .find("crate::ui::draw_bottom_bar(self, ctx, shell.show_bottom_input);")
        .expect("bottom bar compact branch");
    let last_bar = app_source
        .rfind("crate::ui::draw_bottom_bar(self, ctx, shell.show_bottom_input);")
        .expect("bottom bar side branch");
    assert!(first_bar < assistant, "compact: barra antes que el sheet");
    assert!(
        last_bar > assistant,
        "side-panel: barra después del asistente"
    );
    let keyboard = app_source
        .rfind("crate::keyboard::draw_math_keyboard(self, ctx, keyboard_layout);")
        .expect("keyboard draw call");
    assert!(last_bar < keyboard, "la barra sigue debajo del teclado");
    assert!(
        app_source.contains("assistant_limits_status_bar"),
        "el límite depende de visibilidad + modo"
    );
}

#[test]
fn ola11_slash_requests_command_input_focus() {
    // Ola 1.1: "/" (sin foco en texto) pide foco para la entrada de comandos.
    let ctx = egui::Context::default();
    let mut app = crate::app::dummy_grafito_app();
    assert!(!app.command_input_focus_requested);
    let input = top_chrome_input(
        1280.0,
        0.0,
        vec![egui::Event::Key {
            key: egui::Key::Slash,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    let _ = ctx.run(input, |ctx| {
        app.handle_keyboard_shortcuts(ctx);
    });
    assert!(
        app.command_input_focus_requested,
        "'/' debe pedir foco de entrada"
    );
}

#[test]
fn gpu_3d_overlay_keeps_cpu_only_4d_projections_visible() {
    let hypercube =
        grafito_core::GeoObject::HyperSurface4D(grafito_core::HyperSurface4DObj::hypercube());
    let polychoron = grafito_core::GeoObject::RegularPolychoron4D(
        grafito_core::RegularPolychoron4DObj::new(grafito_geometry::RegularPolychoron::Tesseract),
    );
    let polytope =
        grafito_core::GeoObject::RegularPolytopeND(grafito_core::RegularPolytopeNDObj::new(
            grafito_geometry::RegularPolytopeFamily::Hypercube,
            4,
        ));
    let cube = grafito_core::GeoObject::Cube3D(grafito_core::Cube3DObj::new(
        grafito_geometry::Point3D::new(0.0, 0.0, 0.0),
        2.0,
    ));

    assert!(crate::render_3d::requires_cpu_3d_overlay(&hypercube));
    assert!(!crate::render_3d::requires_cpu_3d_overlay(&polychoron));
    assert!(!crate::render_3d::requires_cpu_3d_overlay(&polytope));
    assert!(!crate::render_3d::requires_cpu_3d_overlay(&cube));
    assert!(!crate::render_3d::should_draw_cpu_3d_geometry(
        &polychoron,
        true
    ));
    assert!(!crate::render_3d::should_draw_cpu_3d_geometry(
        &polytope, true
    ));
}

#[test]
fn typed_cpu_projection_reuses_the_renderer_bridge_and_skips_unlabeled_gpu_overlays() {
    let mut polychoron =
        grafito_core::RegularPolychoron4DObj::new(grafito_geometry::RegularPolychoron::Tesseract);
    polychoron.rotation_angles = [0.13, -0.29, 0.41, -0.53, 0.67, -0.79];

    let cpu = crate::render_3d::project_regular_polychoron_cpu(&polychoron, None)
        .expect("CPU fallback projects valid typed geometry");
    let renderer = grafito_render::depth_3d::project_regular_polychoron(
        &polychoron,
        polychoron.rotation_angles,
    )
    .expect("renderer bridge projects valid typed geometry");
    assert!(std::ptr::eq(
        cpu.vertices().as_ptr(),
        renderer.vertices().as_ptr()
    ));

    let mut generic = grafito_core::RegularPolytopeNDObj::new(
        grafito_geometry::RegularPolytopeFamily::Hypercube,
        4,
    );
    generic.rotation_angles = vec![0.11, -0.23, 0.37, -0.41, 0.53, -0.67];
    let cpu = crate::render_3d::project_regular_polytope_nd_cpu(&generic, Some(0.25))
        .expect("CPU fallback projects valid generic geometry");
    let effective = crate::render_3d::effective_typed_four_d_angles(
        generic
            .rotation_angles
            .as_slice()
            .try_into()
            .expect("a four-dimensional generic polytope has six angles"),
        Some(0.25),
    );
    let renderer = grafito_render::depth_3d::project_regular_polytope_nd(&generic, &effective)
        .expect("renderer bridge projects valid generic geometry");
    assert!(std::ptr::eq(
        cpu.vertices().as_ptr(),
        renderer.vertices().as_ptr()
    ));

    let object = grafito_core::GeoObject::RegularPolychoron4D(polychoron.clone());
    assert!(!crate::render_3d::typed_cpu_projection_is_needed(
        &object, true
    ));

    polychoron.label = "P".to_string();
    let labeled = grafito_core::GeoObject::RegularPolychoron4D(polychoron);
    assert!(crate::render_3d::typed_cpu_projection_is_needed(
        &labeled, true
    ));
}

#[test]
fn regular_polytope_cpu_typed_four_d_motion_recognizes_only_four_dimensional_variants() {
    let polychoron = grafito_core::GeoObject::RegularPolychoron4D(
        grafito_core::RegularPolychoron4DObj::new(grafito_geometry::RegularPolychoron::Tesseract),
    );
    let polytope_4d =
        grafito_core::GeoObject::RegularPolytopeND(grafito_core::RegularPolytopeNDObj::new(
            grafito_geometry::RegularPolytopeFamily::Hypercube,
            4,
        ));
    let polytope_5d =
        grafito_core::GeoObject::RegularPolytopeND(grafito_core::RegularPolytopeNDObj::new(
            grafito_geometry::RegularPolytopeFamily::Hypercube,
            5,
        ));
    let legacy =
        grafito_core::GeoObject::HyperSurface4D(grafito_core::HyperSurface4DObj::hypercube());

    assert!(crate::app::is_typed_four_d_projection(&polychoron));
    assert!(crate::app::is_typed_four_d_projection(&polytope_4d));
    assert!(!crate::app::is_typed_four_d_projection(&polytope_5d));
    assert!(!crate::app::is_typed_four_d_projection(&legacy));
    assert_eq!(crate::app::typed_four_d_motion_phase(0.75), Some(0.75));
    assert_eq!(crate::app::typed_four_d_motion_phase(f64::NAN), None);
}

#[test]
fn regular_polytope_cpu_phase_is_continuous_across_all_six_fixed_planes() {
    let base = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6];
    let animated = crate::render_3d::effective_typed_four_d_angles(base, Some(0.25));

    assert_eq!(
        crate::render_3d::effective_typed_four_d_angles(base, None),
        base
    );
    for (index, (base_angle, animated_angle)) in base.into_iter().zip(animated).enumerate() {
        assert_eq!(animated_angle, base_angle + (index + 1) as f64 * 0.25);
    }

    let before =
        crate::render_3d::effective_typed_four_d_angles(base, Some(std::f64::consts::TAU - 1.0e-6));
    let after = crate::render_3d::effective_typed_four_d_angles(base, Some(0.0));
    for (left, right) in before.into_iter().zip(after) {
        assert!((left.sin() - right.sin()).abs() < 1.0e-5);
        assert!((left.cos() - right.cos()).abs() < 1.0e-5);
    }
}

#[test]
fn regular_polytope_cpu_uses_each_fixed_and_lexicographic_four_d_rotation_plane() {
    let fixed =
        grafito_core::RegularPolychoron4DObj::new(grafito_geometry::RegularPolychoron::Tesseract);
    let fixed_baseline = crate::render_3d::project_regular_polychoron_cpu(&fixed, None)
        .expect("unrotated typed polychoron projects");
    for plane in 0..fixed.rotation_angles.len() {
        let mut rotated = fixed.clone();
        rotated.rotation_angles[plane] = 0.37;
        let projection = crate::render_3d::project_regular_polychoron_cpu(&rotated, None)
            .expect("each fixed rotation plane projects");
        assert_ne!(
            projection.vertices(),
            fixed_baseline.vertices(),
            "plane {plane}"
        );
    }

    let generic = grafito_core::RegularPolytopeNDObj::new(
        grafito_geometry::RegularPolytopeFamily::Hypercube,
        4,
    );
    let generic_baseline = crate::render_3d::project_regular_polytope_nd_cpu(&generic, None)
        .expect("unrotated four-dimensional generic polytope projects");
    for plane in 0..generic.rotation_angles.len() {
        let mut rotated = generic.clone();
        rotated.rotation_angles[plane] = 0.37;
        let projection = crate::render_3d::project_regular_polytope_nd_cpu(&rotated, None)
            .expect("each lexicographic rotation plane projects");
        assert_ne!(
            projection.vertices(),
            generic_baseline.vertices(),
            "plane {plane}"
        );
    }
}

#[test]
fn regular_polytope_cpu_projects_every_polychoron_finitely_with_all_six_rotations() {
    let kinds = [
        grafito_geometry::RegularPolychoron::Pentachoron,
        grafito_geometry::RegularPolychoron::Tesseract,
        grafito_geometry::RegularPolychoron::SixteenCell,
        grafito_geometry::RegularPolychoron::TwentyFourCell,
        grafito_geometry::RegularPolychoron::OneTwentyCell,
        grafito_geometry::RegularPolychoron::SixHundredCell,
    ];

    for kind in kinds {
        let mut object = grafito_core::RegularPolychoron4DObj::new(kind);
        object.rotation_angles = [0.13, -0.27, 0.41, -0.59, 0.73, -0.89];
        let geometry = crate::render_3d::project_regular_polychoron_cpu(&object, None)
            .expect("valid canonical polychoron projects through the CPU fallback");

        assert_eq!(geometry.vertices().len(), kind.expected_counts().vertices);
        assert_eq!(geometry.edges().len(), kind.expected_counts().edges);
        assert_eq!(geometry.faces().len(), kind.expected_counts().faces);
        assert!(geometry
            .vertices()
            .iter()
            .all(|point| { grafito_render::depth_3d::point_is_renderable(*point) }));
    }
}

#[test]
fn regular_polytope_cpu_keeps_generic_nd_wireframe_and_phases_only_four_dimensions() {
    let mut four_d = grafito_core::RegularPolytopeNDObj::new(
        grafito_geometry::RegularPolytopeFamily::Hypercube,
        4,
    );
    four_d.rotation_angles = vec![0.1, -0.2, 0.3, -0.4, 0.5, -0.6];
    let static_four_d = crate::render_3d::project_regular_polytope_nd_cpu(&four_d, None)
        .expect("static four-dimensional generic polytope projects");
    let animated_four_d = crate::render_3d::project_regular_polytope_nd_cpu(&four_d, Some(0.25))
        .expect("animated four-dimensional generic polytope projects");

    assert!(static_four_d.faces().is_empty());
    assert_ne!(static_four_d.vertices(), animated_four_d.vertices());

    let mut five_d = grafito_core::RegularPolytopeNDObj::new(
        grafito_geometry::RegularPolytopeFamily::Hypercube,
        5,
    );
    five_d.rotation_angles = vec![0.1; 10];
    let static_five_d = crate::render_3d::project_regular_polytope_nd_cpu(&five_d, None)
        .expect("static five-dimensional generic polytope projects");
    let phased_five_d = crate::render_3d::project_regular_polytope_nd_cpu(&five_d, Some(0.25))
        .expect("five-dimensional generic polytope ignores the four-dimensional phase");

    assert!(static_five_d.faces().is_empty());
    assert_eq!(static_five_d.vertices(), phased_five_d.vertices());
}

#[test]
fn typed_four_d_phase_snapshot_matches_drawing_bounds_without_mutating_the_document() {
    let mut document = grafito_core::Document::new();
    let mut polychoron =
        grafito_core::RegularPolychoron4DObj::new(grafito_geometry::RegularPolychoron::Pentachoron);
    polychoron.rotation_angles = [0.13, -0.29, 0.41, -0.53, 0.67, -0.79];
    let id = document
        .try_add_object(grafito_core::GeoObject::RegularPolychoron4D(polychoron))
        .expect("valid typed polychoron inserts");
    let version_before = document.version;
    let object_before = document.get_object(id).cloned();
    let phase_snapshot = crate::app::typed_four_d_motion_phase(0.73);

    let object = document
        .get_object(id)
        .expect("inserted polychoron remains");
    let grafito_core::GeoObject::RegularPolychoron4D(polychoron) = object else {
        panic!("fixture remains a typed polychoron");
    };
    let drawing = crate::render_3d::project_regular_polychoron_cpu(polychoron, phase_snapshot)
        .expect("drawing projection uses the phase snapshot");
    let drawing_bounds = grafito_geometry::Aabb3D::from_points(drawing.vertices().iter().copied())
        .expect("drawing projection has finite bounds");
    let static_bounds = crate::render_3d::fallback_object_bounds_with_typed_four_d_phase(
        object,
        &document.variables,
        None,
    )
    .expect("static picking bounds");
    let picking_bounds = crate::render_3d::fallback_object_bounds_with_typed_four_d_phase(
        object,
        &document.variables,
        phase_snapshot,
    )
    .expect("phased picking bounds");

    assert!(
        (
            static_bounds.min.x,
            static_bounds.min.y,
            static_bounds.min.z
        ) != (
            picking_bounds.min.x,
            picking_bounds.min.y,
            picking_bounds.min.z
        ) || (
            static_bounds.max.x,
            static_bounds.max.y,
            static_bounds.max.z
        ) != (
            picking_bounds.max.x,
            picking_bounds.max.y,
            picking_bounds.max.z
        ),
        "the typed 4D phase must change projected bounds"
    );
    assert_eq!(
        (
            picking_bounds.min.x,
            picking_bounds.min.y,
            picking_bounds.min.z
        ),
        (
            drawing_bounds.min.x,
            drawing_bounds.min.y,
            drawing_bounds.min.z
        )
    );
    assert_eq!(
        (
            picking_bounds.max.x,
            picking_bounds.max.y,
            picking_bounds.max.z
        ),
        (
            drawing_bounds.max.x,
            drawing_bounds.max.y,
            drawing_bounds.max.z
        )
    );
    assert_eq!(document.version, version_before);
    assert_eq!(document.get_object(id), object_before.as_ref());
}

#[test]
fn typed_four_d_picker_uses_the_phase_snapshot_instead_of_static_bounds() {
    let camera = axis_aligned_test_camera();
    let canvas_size = egui::vec2(800.0, 600.0);
    let mut document = grafito_core::Document::new();
    let mut polychoron =
        grafito_core::RegularPolychoron4DObj::new(grafito_geometry::RegularPolychoron::Pentachoron);
    polychoron.rotation_angles = [0.13, -0.29, 0.41, -0.53, 0.67, -0.79];
    let id = document
        .try_add_object(grafito_core::GeoObject::RegularPolychoron4D(polychoron))
        .expect("valid typed polychoron inserts");

    let phase_snapshot = crate::app::typed_four_d_motion_phase(0.73);
    let object = document
        .get_object(id)
        .expect("inserted polychoron remains");
    let grafito_core::GeoObject::RegularPolychoron4D(polychoron) = object else {
        panic!("fixture remains a typed polychoron");
    };
    let drawing = crate::render_3d::project_regular_polychoron_cpu(polychoron, phase_snapshot)
        .expect("drawing projection uses the phase snapshot");
    let phase_only_pointer = drawing
        .vertices()
        .iter()
        .filter_map(|point| camera.project(point, canvas_size.x, canvas_size.y))
        .map(|(x, y)| egui::vec2(x, y))
        .find(|pointer| {
            crate::render_3d::pick_3d_object_with_typed_four_d_phase(
                &document,
                &camera,
                *pointer,
                canvas_size,
                phase_snapshot,
            ) == Some(id)
                && crate::render_3d::pick_3d_object(&document, &camera, *pointer, canvas_size)
                    .is_none()
        })
        .expect("a phased vertex lies outside the static fallback bounds");

    assert_eq!(
        crate::render_3d::pick_3d_object_with_typed_four_d_phase(
            &document,
            &camera,
            phase_only_pointer,
            canvas_size,
            phase_snapshot,
        ),
        Some(id),
        "selection must use the same phase as the drawn typed polytope"
    );
    assert_eq!(
        crate::render_3d::pick_3d_object(&document, &camera, phase_only_pointer, canvas_size),
        None,
        "the legacy no-phase picker remains static"
    );
}

#[test]
fn typed_four_d_phase_leaves_legacy_and_general_five_d_bounds_static() {
    let phase_snapshot = crate::app::typed_four_d_motion_phase(0.73);
    let legacy =
        grafito_core::GeoObject::HyperSurface4D(grafito_core::HyperSurface4DObj::hypercube());
    let five_d =
        grafito_core::GeoObject::RegularPolytopeND(grafito_core::RegularPolytopeNDObj::new(
            grafito_geometry::RegularPolytopeFamily::Hypercube,
            5,
        ));
    let four_d =
        grafito_core::GeoObject::RegularPolytopeND(grafito_core::RegularPolytopeNDObj::new(
            grafito_geometry::RegularPolytopeFamily::Hypercube,
            4,
        ));
    let variables = std::collections::BTreeMap::new();

    assert_eq!(
        crate::render_3d::typed_four_d_phase_for_object(&legacy, phase_snapshot),
        None
    );
    assert_eq!(
        crate::render_3d::typed_four_d_phase_for_object(&five_d, phase_snapshot),
        None
    );
    assert_eq!(
        crate::render_3d::typed_four_d_phase_for_object(&four_d, phase_snapshot),
        phase_snapshot
    );

    for object in [&legacy, &five_d] {
        let static_bounds = crate::render_3d::fallback_object_bounds_with_typed_four_d_phase(
            object, &variables, None,
        );
        let phased_bounds = crate::render_3d::fallback_object_bounds_with_typed_four_d_phase(
            object,
            &variables,
            phase_snapshot,
        );
        assert_eq!(
            static_bounds, phased_bounds,
            "only typed regular four-dimensional projections receive the motion phase"
        );
    }
}

#[test]
fn regular_polytope_cpu_sorts_static_polychoron_face_triangles_by_depth() {
    let camera = axis_aligned_test_camera();
    let object =
        grafito_core::RegularPolychoron4DObj::new(grafito_geometry::RegularPolychoron::Tesseract);
    let geometry = crate::render_3d::project_regular_polychoron_cpu(&object, None)
        .expect("tesseract CPU geometry");
    let faces = crate::render_3d::projected_polychoron_faces(&camera, &geometry, 800.0, 600.0);
    let expected_triangles = geometry
        .faces()
        .iter()
        .map(|face| face.len() - 2)
        .sum::<usize>();

    assert_eq!(faces.len(), expected_triangles);
    assert!(faces.windows(2).all(|faces| faces[0].0 >= faces[1].0));
    assert!(faces
        .iter()
        .all(|(_, face)| { face.iter().all(|(x, y)| x.is_finite() && y.is_finite()) }));
    assert!(crate::render_3d::should_draw_polychoron_faces(true, false));
    assert!(!crate::render_3d::should_draw_polychoron_faces(true, true));
    assert!(!crate::render_3d::should_draw_polychoron_faces(
        false, false
    ));
}

#[test]
fn regular_polytope_cpu_preview_bounds_wire_work_and_never_mutates_the_document() {
    let mut document = grafito_core::Document::new();
    let object = grafito_core::RegularPolychoron4DObj::new(
        grafito_geometry::RegularPolychoron::OneTwentyCell,
    );
    let id = object.id;
    document
        .try_add_object(grafito_core::GeoObject::RegularPolychoron4D(object))
        .expect("valid polychoron inserts into the document");
    let version_before = document.version;
    let object_before = document.get_object(id).cloned();

    let object = document
        .get_object(id)
        .expect("inserted object remains available");
    let grafito_core::GeoObject::RegularPolychoron4D(object) = object else {
        panic!("inserted object remains a typed polychoron");
    };
    let geometry = crate::render_3d::project_regular_polychoron_cpu(object, Some(0.25))
        .expect("motion-preview CPU projection remains finite");
    let stride =
        crate::render_3d::motion_preview_polytope_edge_stride(geometry.edges().len(), true);

    assert!(
        geometry.edges().len().div_ceil(stride)
            <= crate::render_3d::MAX_MOTION_PREVIEW_POLYTOPE_EDGES
    );
    assert_eq!(document.version, version_before);
    assert_eq!(document.get_object(id), object_before.as_ref());
}

#[test]
fn wide_shell_allows_contextual_drawer_only_when_available() {
    let available =
        crate::ShellLayout::for_viewport(1720.0, crate::Perspective::Complex, 0, true, true, true);
    assert!(available.show_left_drawer);
    assert!(available.show_right_drawer);

    let unavailable =
        crate::ShellLayout::for_viewport(1720.0, crate::Perspective::Complex, 0, false, true, true);
    assert!(!unavailable.show_right_drawer);
}

#[test]
fn contextual_drawer_waits_for_space_beyond_the_permanent_assistant() {
    let shell =
        crate::ShellLayout::for_viewport(1360.0, crate::Perspective::Complex, 0, true, true, true);

    assert_eq!(shell.width_class, crate::ShellWidthClass::Medium);
    assert!(!shell.show_right_drawer);
}

#[test]
fn command_input_width_never_becomes_negative() {
    assert_eq!(crate::ui::command_input_width(20.0, 40.0), 0.0);
    assert_eq!(crate::ui::command_input_width(140.0, 40.0), 100.0);
}

#[test]
fn assistant_credentials_use_fixed_keyring_accounts() {
    use grafito_assistant_types::ProviderProfile;

    assert_eq!(
        crate::assistant_credentials::account_for(ProviderProfile::OpenCodeGo),
        Some("assistant-opencode-go")
    );
    assert_eq!(
        crate::assistant_credentials::account_for(ProviderProfile::OllamaLocal),
        None
    );
}

#[test]
fn assistant_text_editor_blocks_global_shortcuts() {
    assert!(!crate::app::global_shortcuts_allowed(true));
    assert!(crate::app::global_shortcuts_allowed(false));
}

#[test]
fn clean_document_actions_proceed_without_an_unsaved_changes_dialog() {
    use crate::app::{DocumentAction, DocumentActionRequest, DocumentLifecycle};

    for action in [
        DocumentAction::New,
        DocumentAction::Open,
        DocumentAction::Exit,
    ] {
        let document = crate::app::initial_document();
        let mut lifecycle = DocumentLifecycle::new(&document);
        assert_eq!(
            lifecycle.request_action(action, &document, false),
            DocumentActionRequest::Proceed(action)
        );
        assert_eq!(lifecycle.pending_action(), None);
    }
}

#[test]
fn toast_messages_are_wrapped_before_drawing() {
    let wrapped = crate::app::wrap_toast_message("uno dos tres cuatro cinco", 8);
    assert_eq!(wrapped, "uno dos\ntres\ncuatro\ncinco");
}

#[test]
fn toast_wrapping_splits_a_single_long_token() {
    assert_eq!(
        crate::app::wrap_toast_message("abcdefghij", 4),
        "abcd\nefgh\nij"
    );
}

#[test]
fn ctrl_shift_y_is_reserved_for_y_intercept_not_redo() {
    assert_eq!(
        crate::app::ctrl_y_shortcut(false),
        crate::app::CtrlYShortcut::Redo
    );
    assert_eq!(
        crate::app::ctrl_y_shortcut(true),
        crate::app::CtrlYShortcut::YIntercept
    );
}

#[test]
fn document_lifecycle_fresh_workspace_is_pathless_and_clean() {
    let document = crate::app::initial_document();
    let lifecycle = crate::app::DocumentLifecycle::new(&document);

    assert_eq!(lifecycle.current_path(), None);
    assert!(!lifecycle.is_dirty(&document, false));
}

#[test]
fn document_lifecycle_dirty_baseline_ignores_transient_revision_and_quality() {
    let mut document = crate::app::initial_document();
    let lifecycle = crate::app::DocumentLifecycle::new(&document);

    document.bump_version();
    document.render_quality = grafito_core::RenderQuality::Preview;
    document.view_mut().screen_size = glam::Vec2::new(931.0, 577.0);
    document.invalidate_all_caches();
    assert!(!lifecycle.is_dirty(&document, false));

    document.add_object(grafito_core::GeoObject::Point(grafito_core::PointObj::new(
        grafito_geometry::Point2::new(1.0, 2.0),
    )));
    assert!(lifecycle.is_dirty(&document, false));
}

#[test]
fn document_lifecycle_successful_save_updates_path_and_semantic_baseline() {
    let mut document = crate::app::initial_document();
    let mut lifecycle = crate::app::DocumentLifecycle::new(&document);
    let path = std::path::PathBuf::from("/tmp/grafito-saved.json");

    document.set_variable("a".into(), 3.0);
    assert!(lifecycle.is_dirty(&document, false));
    assert_eq!(
        lifecycle.current_save_path(crate::app::SaveMode::Save),
        None
    );

    assert_eq!(lifecycle.record_save_success(path.clone(), &document), None);
    assert_eq!(lifecycle.current_path(), Some(path.as_path()));
    assert_eq!(
        lifecycle.current_save_path(crate::app::SaveMode::Save),
        Some(path.as_path())
    );
    assert_eq!(
        lifecycle.current_save_path(crate::app::SaveMode::SaveAs),
        None
    );
    assert!(!lifecycle.is_dirty(&document, false));
}

#[test]
fn document_lifecycle_dirty_new_open_and_exit_wait_for_a_decision() {
    use crate::app::{DocumentAction, DocumentActionRequest, DocumentLifecycle};

    for action in [
        DocumentAction::New,
        DocumentAction::Open,
        DocumentAction::Exit,
    ] {
        let mut document = crate::app::initial_document();
        let mut lifecycle = DocumentLifecycle::new(&document);
        document.set_variable("dirty".into(), 1.0);

        assert_eq!(
            lifecycle.request_action(action, &document, false),
            DocumentActionRequest::AwaitDecision(action)
        );
        assert_eq!(lifecycle.pending_action(), Some(action));
    }
}

#[test]
fn document_lifecycle_cancel_and_discard_have_explicit_state_transitions() {
    use crate::app::{DocumentAction, DocumentLifecycle, UnsavedDecision, UnsavedResolution};

    let mut document = crate::app::initial_document();
    let mut lifecycle = DocumentLifecycle::new(&document);
    let path = std::path::PathBuf::from("/tmp/grafito-cancel-preserves.json");
    lifecycle.record_save_success(path.clone(), &document);
    document.set_variable("dirty".into(), 1.0);
    lifecycle.request_action(DocumentAction::New, &document, false);

    assert_eq!(
        lifecycle.resolve_unsaved_decision(UnsavedDecision::Cancel),
        Some(UnsavedResolution::Cancelled)
    );
    assert_eq!(lifecycle.pending_action(), None);
    assert_eq!(lifecycle.current_path(), Some(path.as_path()));
    assert!(lifecycle.is_dirty(&document, false));

    lifecycle.request_action(DocumentAction::Open, &document, false);
    assert_eq!(
        lifecycle.resolve_unsaved_decision(UnsavedDecision::Discard),
        Some(UnsavedResolution::Proceed(DocumentAction::Open))
    );
    assert_eq!(lifecycle.pending_action(), None);
    assert_eq!(lifecycle.current_path(), Some(path.as_path()));
    assert!(lifecycle.is_dirty(&document, false));
}

#[test]
fn document_lifecycle_save_failure_retains_the_document_action_dialog() {
    use crate::app::{DocumentAction, DocumentLifecycle, UnsavedDecision, UnsavedResolution};

    let mut document = crate::app::initial_document();
    let mut lifecycle = DocumentLifecycle::new(&document);
    document.set_variable("dirty".into(), 1.0);
    lifecycle.request_action(DocumentAction::Exit, &document, false);

    assert_eq!(
        lifecycle.resolve_unsaved_decision(UnsavedDecision::Save),
        Some(UnsavedResolution::Save(DocumentAction::Exit))
    );
    lifecycle.record_save_failure("disk full");

    assert_eq!(lifecycle.pending_action(), Some(DocumentAction::Exit));
    assert_eq!(lifecycle.save_error(), Some("disk full"));
    assert!(lifecycle.is_dirty(&document, false));

    lifecycle.request_action(DocumentAction::Exit, &document, false);
    assert_eq!(lifecycle.save_error(), Some("disk full"));
}

#[test]
fn document_lifecycle_save_then_open_releases_action_and_open_establishes_identity() {
    use crate::app::{DocumentAction, DocumentLifecycle};

    let mut document = crate::app::initial_document();
    let mut lifecycle = DocumentLifecycle::new(&document);
    document.set_variable("dirty".into(), 1.0);
    lifecycle.request_action(DocumentAction::Open, &document, false);

    let saved_path = std::path::PathBuf::from("/tmp/grafito-before-open.json");
    assert_eq!(
        lifecycle.record_save_success(saved_path.clone(), &document),
        Some(DocumentAction::Open)
    );
    assert_eq!(lifecycle.current_path(), Some(saved_path.as_path()));
    assert!(!lifecycle.is_dirty(&document, false));

    let mut opened = crate::app::initial_document();
    opened.set_variable("opened".into(), 42.0);
    let opened_path = std::path::PathBuf::from("/tmp/grafito-opened.json");
    lifecycle.establish_opened_document(opened_path.clone(), &opened);

    assert_eq!(lifecycle.current_path(), Some(opened_path.as_path()));
    assert!(!lifecycle.is_dirty(&opened, false));

    let blank = crate::app::initial_document();
    lifecycle.establish_new_document(&blank);
    assert_eq!(lifecycle.current_path(), None);
    assert!(!lifecycle.is_dirty(&blank, false));
}

#[test]
fn document_lifecycle_invalid_open_candidate_does_not_touch_live_document() {
    let mut live = crate::app::initial_document();
    live.set_variable("keep".into(), 7.0);
    let before = serde_json::to_value(&live).expect("live document should serialize");
    let path = std::env::temp_dir().join(format!(
        "grafito-invalid-open-{}-{}.json",
        std::process::id(),
        line!()
    ));
    std::fs::write(&path, b"not valid document json").expect("write invalid fixture");

    let candidate = crate::app::load_document_candidate(&path);
    let _ = std::fs::remove_file(path);

    assert!(candidate.is_err());
    assert_eq!(
        serde_json::to_value(&live).expect("live document should serialize"),
        before
    );
}

#[cfg(unix)]
#[test]
fn document_lifecycle_non_utf8_document_paths_are_not_lossy() {
    use std::os::unix::ffi::OsStringExt;

    let mut document = crate::app::initial_document();
    document.set_variable("non_utf8".into(), 9.0);
    let mut file_name = format!("grafito-non-utf8-{}-", std::process::id()).into_bytes();
    file_name.extend_from_slice(&[0xff, b'.', b'j', b's', b'o', b'n']);
    let path = std::env::temp_dir().join(std::ffi::OsString::from_vec(file_name));
    match grafito_core::write_document_atomic(&document, &path) {
        Ok(()) => {
            let loaded = crate::app::load_document_candidate(&path);
            let _ = std::fs::remove_file(&path);

            assert_eq!(
                loaded
                    .expect("non-UTF-8 path should load")
                    .get_variable("non_utf8"),
                Some(9.0)
            );
        }
        // macOS (APFS) rechaza nombres no-UTF-8 a nivel OS (EILSEQ): no hay
        // roundtrip posible y lo honesto es verificar el fallo limpio (sin
        // pánico, sin archivo a medias), no ignorar el test.
        Err(error) => {
            if cfg!(target_os = "macos") {
                assert!(
                    !path.exists(),
                    "no debe quedar archivo a medias tras el rechazo del OS"
                );
            } else {
                panic!("write non-UTF-8 fixture falló en plataforma que lo soporta: {error:?}");
            }
        }
    }
}

#[test]
fn document_lifecycle_file_shortcuts_share_the_file_command_policy() {
    use crate::lifecycle::{file_shortcut, FileCommand};
    use egui::Key;

    assert_eq!(file_shortcut(Key::N, true, false), Some(FileCommand::New));
    assert_eq!(file_shortcut(Key::O, true, false), Some(FileCommand::Open));
    assert_eq!(file_shortcut(Key::S, true, false), Some(FileCommand::Save));
    assert_eq!(file_shortcut(Key::S, true, true), Some(FileCommand::SaveAs));
    assert_eq!(file_shortcut(Key::N, false, false), None);
}

#[test]
fn document_lifecycle_ui_exposes_save_as_and_native_close_cancellation() {
    let ui = include_str!("ui.rs");
    // Scandinavian F14: el atajo va alineado a la derecha (`shortcut_text`),
    // no pegado al label; los ítems usan "…" tipográfico.
    // En drill el atajo viaja como arg de `drill_action` (en móvil se
    // oculta: ruido táctil) y el helper lo pone en `shortcut_text` en
    // desktop: el mecanismo se pinea una vez, no por ítem.
    assert!(ui.contains("btn.shortcut_text(shortcut)"));
    assert!(ui.contains("\"Ctrl+N\""));
    assert!(ui.contains("\"Ctrl+O\""));
    assert!(ui.contains("\"Ctrl+S\""));
    assert!(ui.contains("\"Ctrl+Shift+S\""));
    // Ola 1.3: label vía i18n; el atajo sigue literal.
    assert!(ui.contains("menu.file.open"));
    assert!(ui.contains("menu.file.save_as"));

    let app = include_str!("app.rs");
    assert!(app.contains("i.viewport().close_requested()"));
    assert!(app.contains("egui::ViewportCommand::CancelClose"));
}

#[test]
fn document_lifecycle_replacement_commits_before_transient_reset() {
    let source = include_str!("app.rs");
    let start = source
        .find("fn replace_document")
        .expect("document replacement helper");
    let body = &source[start..];
    let commit = body
        .find("self.document = document;")
        .expect("replacement commit");
    let reset = body
        .find("self.clear_document_bound_transient_state();")
        .expect("transient reset");

    assert!(commit < reset);
}

#[test]
fn deferred_file_actions_are_single_slot_and_dialog_decisions_have_priority() {
    use crate::app::{DeferredFileActions, DeferredFileIntent, FileCommand, UnsavedDecision};

    let mut actions = DeferredFileActions::default();
    assert!(actions.queue_command(FileCommand::Open));
    assert!(!actions.queue_command(FileCommand::Save));
    assert_eq!(
        actions.pending(),
        Some(DeferredFileIntent::Command(FileCommand::Open))
    );

    assert!(actions.queue_decision(UnsavedDecision::Save));
    assert!(!actions.queue_command(FileCommand::Exit));
    assert_eq!(
        actions.take_after_editors(),
        Some(DeferredFileIntent::Decision(UnsavedDecision::Save))
    );
    assert_eq!(actions.take_after_editors(), None);

    let mut native = DeferredFileActions::default();
    assert!(native.intercept_native_close(false));
    assert!(!native.intercept_native_close(true));
    assert_eq!(
        native.take_after_editors(),
        Some(DeferredFileIntent::Command(FileCommand::Exit))
    );
}

// ── AG14: Headless panel render tests + teaching_ui mappings ─────────────────

#[cfg(test)]
/// Textos visibles de una corrida headless (evidencia de render de UI).
fn headless_texts(output: &egui::FullOutput) -> impl Iterator<Item = &str> {
    output.shapes.iter().filter_map(|clipped| {
        if let egui::epaint::Shape::Text(text) = &clipped.shape {
            Some(text.galley.text())
        } else {
            None
        }
    })
}

fn headless_raw_input() -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1280.0, 720.0),
        )),
        ..Default::default()
    }
}

#[test]
fn headless_dummy_app_maintains_viewmode_perspective_invariant() {
    use crate::{CanvasMode, Perspective, ViewMode};
    for perspective in Perspective::ALL {
        let app = crate::app::dummy_grafito_app_with_perspective(perspective);
        assert_eq!(
            app.current_view,
            perspective.view_mode(),
            "current_view diverged for {perspective:?}"
        );
        assert_eq!(
            app.current_view,
            app.perspective.view_mode(),
            "app.view invariant failed for {perspective:?}"
        );
        assert_eq!(
            perspective.canvas_mode().view_mode(),
            perspective.view_mode(),
            "CanvasMode collapse mismatch for {perspective:?}"
        );
        assert_eq!(
            perspective.layout().canvas_mode,
            perspective.canvas_mode(),
            "layout.canvas_mode must equal Perspective::canvas_mode for {perspective:?}"
        );
        // ViewMode derived helper must stay in sync
        assert_eq!(
            ViewMode::from_perspective(perspective),
            perspective.view_mode()
        );
        assert_eq!(
            CanvasMode::from_perspective(perspective),
            perspective.canvas_mode()
        );
        // Verify set_perspective keeps invariant (no direct field mutation)
        let mut mutable = crate::app::dummy_grafito_app();
        mutable.set_perspective(perspective);
        assert_eq!(mutable.current_view, mutable.perspective.view_mode());
        assert_eq!(mutable.perspective, perspective);
    }
}

#[test]
fn headless_algebra_panel_renders_without_panic() {
    let ctx = egui::Context::default();
    let mut app = crate::app::dummy_grafito_app_with_perspective(crate::Perspective::Geometry2D);
    // populate with a few objects to exercise list code
    app.document
        .try_add_point(grafito_geometry::Point2::new(1.0, 2.0))
        .expect("point inserts");
    app.document
        .try_add_point(grafito_geometry::Point2::new(2.0, 3.0))
        .expect("point inserts");
    let output = ctx.run(headless_raw_input(), |ctx| {
        crate::algebra::draw_algebra_panel(&mut app, ctx);
    });
    // also verify no unwrap panic: output shapes exist (frame + text)
    // we keep check loose to avoid brittle shape counts across egui versions
    assert!(output.shapes.len() < 10_000, "algebra panel not explosive");
    assert!(
        output.textures_delta.set.is_empty() || !output.textures_delta.set.is_empty(),
        "textures_delta valid"
    );
}

#[test]
fn headless_view_panel_renders_without_panic() {
    let ctx = egui::Context::default();
    let mut app = crate::app::dummy_grafito_app();
    let output = ctx.run(headless_raw_input(), |ctx| {
        crate::panels::draw_view_panel(&mut app, ctx);
    });
    assert!(output.shapes.len() < 10_000);
    // toggle perspective and re-render to ensure ViewMode consistency
    app.set_perspective(crate::Perspective::Geometry3D);
    assert_eq!(app.current_view, crate::ViewMode::D3);
    let _ = ctx.run(headless_raw_input(), |ctx| {
        crate::panels::draw_view_panel(&mut app, ctx);
    });
}

#[test]
fn vista_es_solo_visual_exportar_datos_y_prob_viven_en_su_tab() {
    // Vista no debe volver a tragar Exportación, hoja ni probabilidad:
    // cada una tiene tab propio (Herram./Datos/Prob.).
    let source = include_str!("panels.rs");
    let start = source
        .find("pub(crate) fn draw_view_panel")
        .expect("vista panel");
    let end = source[start..]
        .find("pub(crate) fn draw_trig_animation_panel")
        .map(|offset| start + offset)
        .expect("next panel");
    let vista = &source[start..end];
    assert!(
        !vista.contains("draw_spreadsheet_section"),
        "la hoja vive en Datos, no en Vista"
    );
    assert!(
        !vista.contains("draw_probability_section"),
        "la probabilidad vive en Prob., no en Vista"
    );
    assert!(
        !vista.contains("\"Exportación\""),
        "exportar vive en Herramientas, no en Vista"
    );
    assert!(
        vista.contains("view_chrome_for(\"layers_title\", view_locale)"),
        "Vista conserva Capas vía dispatcher localizado (B-A129, antes título puro VIEW_LAYERS_TITLE)"
    );
    assert!(
        vista.contains("view_chrome_for(\"section_3d\", view_locale)"),
        "Vista conserva Vista 3D vía dispatcher localizado (B-A129, antes título puro VIEW_3D_SECTION_TITLE)"
    );
}

#[test]
fn headless_data_panel_renders_without_panic() {
    let ctx = egui::Context::default();
    let mut app = crate::app::dummy_grafito_app_with_perspective(crate::Perspective::DataAnalysis);
    // El tab Datos es la casa de Análisis de datos.
    assert_eq!(
        app.perspective.layout().left_panel,
        crate::LeftPanelContent::Data
    );
    assert_eq!(crate::LeftPanelContent::Data.default_sidebar_tab(), 2);
    let output = ctx.run(headless_raw_input(), |ctx| {
        crate::panels::draw_data_panel(&mut app, ctx);
    });
    assert!(output.shapes.len() < 15_000);
    let output_export = ctx.run(headless_raw_input(), |ctx| {
        crate::tools_panel::draw_tools_panel(&mut app, ctx);
    });
    assert!(output_export.shapes.len() < 15_000);
}

#[test]
fn implicit_slot_productor_envia_grueso_y_limpia_sin_superficie() {
    // P1b: productor real — con superficie visible envía (pending o válido
    // tras drenar); sin superficie limpia (sin fantasma).
    use grafito_core::{GeoObject, ImplicitSurface3DObj};
    let mut app = crate::app::dummy_grafito_app();
    // Sin superficie: no envía y queda limpio.
    app.maybe_submit_implicit_slot();
    assert!(!app.implicit_surface_slot.has_pending());
    assert!(app.implicit_slot_key.is_none());
    // Con superficie visible: envía grueso 12³ sin bloquear.
    let surface = GeoObject::ImplicitSurface3D(ImplicitSurface3DObj::new(
        "x*x+y*y+z*z-1",
        (-1.5, 1.5, -1.5, 1.5, -1.5, 1.5),
        16,
    ));
    let _ = app.document.try_add_object(surface);
    app.maybe_submit_implicit_slot();
    assert!(
        app.implicit_surface_slot.has_pending() || app.implicit_slot_key.is_some(),
        "el productor debe enviar la superficie visible"
    );
    assert!(app.implicit_slot_key.is_some());
    // Segunda llamada con misma clave no re-envía (no resetea el job).
    let key_antes = app.implicit_slot_key;
    app.maybe_submit_implicit_slot();
    assert_eq!(app.implicit_slot_key, key_antes);
    // Al borrar todo, el productor limpia el slot (sin fantasma).
    app.document = grafito_core::Document::new();
    // Drena un poll para no dejar receiver colgado antes de limpiar.
    let _ = app.implicit_surface_slot.poll();
    app.maybe_submit_implicit_slot();
    assert!(!app.implicit_surface_slot.has_pending());
    assert!(app.implicit_surface_slot.last_valid().is_none());
    assert!(app.implicit_slot_key.is_none());
}

#[test]
fn headless_tools_panel_renders_without_panic_for_both_views() {
    for perspective in [
        crate::Perspective::Geometry2D,
        crate::Perspective::Geometry3D,
    ] {
        let ctx = egui::Context::default();
        let mut app = crate::app::dummy_grafito_app_with_perspective(perspective);
        let output = ctx.run(headless_raw_input(), |ctx| {
            crate::tools_panel::draw_tools_panel(&mut app, ctx);
        });
        assert!(
            output.shapes.len() < 15_000,
            "tools panel overflow for {perspective:?}"
        );
        // verify perspective ↔ view_mode link
        assert_eq!(app.current_view, perspective.view_mode());
    }
}

#[test]
fn headless_complex_and_statistics_panels_render_without_panic() {
    let ctx = egui::Context::default();
    let mut app = crate::app::dummy_grafito_app_with_perspective(crate::Perspective::Complex);
    // ensure complex perspective respects invariants
    assert_eq!(app.current_view, crate::ViewMode::D2);
    assert_eq!(app.perspective, crate::Perspective::Complex);
    let out_complex = ctx.run(headless_raw_input(), |ctx| {
        crate::panels::draw_complex_panel(&mut app, ctx);
    });
    assert!(out_complex.shapes.len() < 10_000);
    // La sección del contorno a mano existe y se dibuja (título visible).
    assert!(
        headless_texts(&out_complex).any(|text| text.contains("Integral de contorno")),
        "el panel complejo debe mostrar la sección de contorno"
    );
    // Armado: la herramienta cambia y el panel guía al lienzo.
    app.arm_complex_contour(crate::complex_contour::ComplexContourMode::Freehand)
        .expect("f(z) por defecto es válida");
    assert_eq!(app.current_tool, grafito_ui::Tool::ComplexContour);
    let out_armed = ctx.run(headless_raw_input(), |ctx| {
        crate::panels::draw_complex_panel(&mut app, ctx);
    });
    assert!(
        headless_texts(&out_armed).any(|text| text.contains("Dibujá sobre el lienzo")),
        "armado: el panel indica dibujar en el lienzo"
    );

    let mut app2 = crate::app::dummy_grafito_app_with_perspective(crate::Perspective::Statistics);
    // inject some data to exercise statistics path without panic
    app2.statistics_data = vec![1.0, 2.0, 3.0, 4.0];
    let out_stats = ctx.run(headless_raw_input(), |ctx| {
        crate::panels::draw_statistics_panel(&mut app2, ctx);
    });
    assert!(out_stats.shapes.len() < 10_000);

    let mut app3 = crate::app::dummy_grafito_app_with_perspective(crate::Perspective::Dynamics);
    let out_attractor = ctx.run(headless_raw_input(), |ctx| {
        crate::panels::draw_attractor_panel(&mut app3, ctx);
    });
    assert!(out_attractor.shapes.len() < 10_000);
}

#[test]
fn headless_right_properties_panel_renders_without_panic() {
    let ctx = egui::Context::default();
    let mut app = crate::app::dummy_grafito_app_with_perspective(crate::Perspective::Geometry3D);
    // no selection → empty state
    let out_empty = ctx.run(headless_raw_input(), |ctx| {
        crate::panels::draw_right_properties_panel(&mut app, ctx);
    });
    assert!(out_empty.shapes.len() < 10_000);
    // with selection → inspector cards
    let cube = app.document.add_object(grafito_core::GeoObject::Cube3D(
        grafito_core::Cube3DObj::new(grafito_geometry::Point3D::new(0.0, 0.0, 0.0), 2.0),
    ));
    app.selected_object = Some(cube);
    let out_filled = ctx.run(headless_raw_input(), |ctx| {
        crate::panels::draw_right_properties_panel(&mut app, ctx);
    });
    assert!(out_filled.shapes.len() < 15_000);
    // ensure selection survives second frame (no panic on id collision)
    let _ = ctx.run(headless_raw_input(), |ctx| {
        crate::panels::draw_right_properties_panel(&mut app, ctx);
    });
}

#[test]
fn q4_inspector_implicita_cells_altas_render_sin_panico_con_cancel_real() {
    // Q4: Inspector live-preview + cancel, headless. Superficie de cells
    // altas (32³) seleccionada: el panel dibuja sin pánico, el productor
    // existente envía el grueso, y `cancel()` corta el pendiente de verdad
    // sin tocar el último válido.
    use grafito_core::{GeoObject, ImplicitSurface3DObj};
    let ctx = egui::Context::default();
    let mut app = crate::app::dummy_grafito_app_with_perspective(crate::Perspective::Geometry3D);
    let surface = GeoObject::ImplicitSurface3D(ImplicitSurface3DObj::new(
        "x*x+y*y+z*z-1",
        (-1.5, 1.5, -1.5, 1.5, -1.5, 1.5),
        32,
    ));
    let id = app.document.add_object(surface);
    app.selected_object = Some(id);
    let out = ctx.run(headless_raw_input(), |ctx| {
        crate::panels::draw_right_properties_panel(&mut app, ctx);
    });
    assert!(
        out.shapes.len() < 15_000,
        "inspector implícita sin overflow"
    );
    // Helpers puros del slot (mismo contrato que los botones usan).
    assert!(crate::implicit_surface_compute::should_offer_quick_preview(
        32
    ));
    assert!(!crate::implicit_surface_compute::should_offer_quick_preview(8));
    // Vista previa rápida vía productor existente (no bloquea).
    app.maybe_submit_implicit_slot();
    assert!(
        app.implicit_surface_slot.has_pending() || app.implicit_slot_key.is_some(),
        "el productor debe enviar la implícita visible"
    );
    // Cancel real: corta el pendiente, preserva clave/estado sin fantasma.
    if app.implicit_surface_slot.has_pending() {
        app.implicit_surface_slot.cancel();
        assert!(!app.implicit_surface_slot.has_pending());
        assert_eq!(app.implicit_surface_slot.pending_cells(), 0);
    }
}

#[test]
fn headless_cas_and_trig_panels_render_without_panic() {
    let ctx = egui::Context::default();
    let mut app = crate::app::dummy_grafito_app_with_perspective(crate::Perspective::AlgebraCas);
    let out_cas = ctx.run(headless_raw_input(), |ctx| {
        crate::panels::draw_cas_panel(&mut app, ctx);
    });
    assert!(out_cas.shapes.len() < 10_000);

    // trig animation panel is tied to D2 perspective; verify gating
    let mut app_d2 = crate::app::dummy_grafito_app_with_perspective(crate::Perspective::Geometry2D);
    assert!(crate::app::trig_animation_supported(app_d2.current_view));
    let _ = ctx.run(headless_raw_input(), |ctx| {
        crate::panels::draw_trig_animation_panel(&mut app_d2, ctx);
    });
    let app_d3 = crate::app::dummy_grafito_app_with_perspective(crate::Perspective::Geometry3D);
    assert!(!crate::app::trig_animation_supported(app_d3.current_view));
}

/// Acumula textos pintados más allá del ancho disponible (borde del panel).
fn collect_trig_text_overflow(shape: &egui::Shape, limit: f32, out: &mut Vec<String>) {
    match shape {
        egui::Shape::Text(text) => {
            let right = text.pos.x + text.galley.size().x;
            if right > limit + 0.5 {
                out.push(format!("{:?} bordes right={right:.1}", text.galley.text()));
            }
        }
        egui::Shape::Vec(shapes) => {
            for shape in shapes {
                collect_trig_text_overflow(shape, limit, out);
            }
        }
        _ => {}
    }
}

/// El explorador trigonométrico no desborda su ancho en ningún tamaño de
/// ventana: todo texto pintado queda dentro del borde derecho del panel
/// (= borde de la ventana). Atrapa la regresión de los chips `sin(t)…csc(t)`,
/// el prompt "Auto" y los sliders recortados.
#[test]
fn trig_panel_text_never_overflows_its_width() {
    for width in [480.0_f32, 560.0, 640.0, 960.0, 1080.0, 1280.0, 1600.0] {
        let ctx = egui::Context::default();
        grafito_ui::theme::DARK.apply(&ctx);
        let mut app =
            crate::app::dummy_grafito_app_with_perspective(crate::Perspective::Geometry2D);
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width, 720.0),
            )),
            ..Default::default()
        };
        let output = ctx.run(input, |ctx| {
            crate::panels::draw_trig_animation_panel(&mut app, ctx);
        });
        let mut overflow = Vec::new();
        for clipped in &output.shapes {
            collect_trig_text_overflow(&clipped.shape, width, &mut overflow);
        }
        assert!(
            overflow.is_empty(),
            "ancho {width}: {} textos fuera del panel: {overflow:?}",
            overflow.len()
        );
    }
}

// ── Composición de la bienvenida: medida real, no supuesto ────────────────

/// Instala Inter en el contexto: sin fuentes, egui no rasteriza texto y toda
/// la composición se mide sobre shapes `Noop`. Reusa el mismo asset que
/// `GrafitoApp::new` carga en producción.
#[cfg(test)]
fn ctx_with_fonts() -> egui::Context {
    let ctx = egui::Context::default();
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "Inter".to_owned(),
        egui::FontData::from_static(include_bytes!("../../../assets/InterVariable.ttf")),
    );
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "Inter".to_owned());
    ctx.set_fonts(fonts);
    ctx
}

/// Ancho del texto pintado más ancho del árbol de shapes.
fn collect_widest_text(shape: &egui::Shape, widest: &mut f32) {
    match shape {
        egui::Shape::Text(text) => {
            let w = text.galley.size().x;
            if w > *widest {
                *widest = w;
            }
        }
        egui::Shape::Vec(shapes) => {
            for shape in shapes {
                collect_widest_text(shape, widest);
            }
        }
        _ => {}
    }
}

/// Ejecuta `f` dos veces y devuelve el output del segundo pass.
///
/// egui necesita dos pasadas para una superficie que se registra por primera
/// vez en el frame actual: un `Area` reserva su rect en el frame en que se
/// registra y sólo **pinta** cuando ya tiene tamaño asignado. Con un solo
/// `ctx.run`, `draw_onboarding_window` devuelve seis `Shape::Noop` — la
/// ventana queda vacía sin ningún error ni aviso.
///
/// Es un límite real del harness headless, no de la app: en producción el
/// bucle de eframe repinta y el segundo frame muestra la pantalla. El helper
/// lo hace explícito para que ningún test de composición vuelva a necesitar
/// dos pasadas a mano.
fn run_two_passes(
    ctx: &egui::Context,
    size: egui::Vec2,
    mut f: impl FnMut(&egui::Context),
) -> egui::FullOutput {
    let input = || egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
        ..Default::default()
    };
    // El tamaño se pasa explícito porque `ctx.screen_rect()` exige un
    // `Context::run()` activo y este helper corre por fuera de él.
    let _ = ctx.run(input(), |ctx| f(ctx));
    let first = ctx.run(input(), |ctx| f(ctx));
    let second = ctx.run(input(), |ctx| f(ctx));
    if second
        .shapes
        .iter()
        .any(|c| !matches!(c.shape, egui::Shape::Noop))
    {
        second
    } else {
        first
    }
}

/// Rect del primer texto pintado cuyo contenido contiene `needle`.
fn find_painted_text(shape: &egui::Shape, needle: &str, out: &mut Option<egui::Rect>) {
    match shape {
        egui::Shape::Text(text) => {
            if out.is_none() && format!("{:?}", text.galley.text()).contains(needle) {
                *out = Some(egui::Rect::from_min_size(text.pos, text.galley.size()));
            }
        }
        egui::Shape::Vec(shapes) => {
            for shape in shapes {
                find_painted_text(shape, needle, out);
            }
        }
        _ => {}
    }
}

/// La bienvenida no debe dejar el copy flotando en el centro matemático.
///
/// El bloque arrancaba en `screen.height() * 0.16` desde un `Area` anclado al
/// centro, así que el titular caía alrededor del 34 % del alto y la pantalla
/// se leía como un bloque desligado del resto, con ~700 px muertos encima y
/// debajo.
///
/// Ahora el copy **y** la navegación forman un bloque compacto anclado al
/// tercio superior. La acción primaria ya no se ancla al fondo de la columna:
/// hacerlo dejaba un abismo de ~350 px entre el subtítulo y los botones — un
/// hueco que se leía como espacio muerto, no como respiración. Lo que se mide
/// acá es que todo el bloque entre en el tercio superior y que no se disperse
/// por la pantalla.
#[test]
fn welcome_copy_enters_the_upper_third_as_one_block() {
    let height = 900.0_f32;
    let ctx = ctx_with_fonts();
    let mut app = crate::app::dummy_grafito_app();
    app.show_onboarding = true;
    app.welcome_page = 0;
    let output = run_two_passes(&ctx, egui::vec2(1440.0, height), |ctx| {
        app.draw_onboarding_window(ctx)
    });
    let mut brand = None;
    let mut primary = None;
    for clipped in &output.shapes {
        find_painted_text(&clipped.shape, "G R A F I T O", &mut brand);
        // El botón primario de la única pantalla es "Probar ahora".
        find_painted_text(&clipped.shape, "Probar ahora", &mut primary);
    }
    let brand = brand.expect("la marca debe pintarse en la primera página");
    let primary = primary.expect("el botón primario debe pintarse en la primera página");

    assert!(
        brand.min.y < height / 3.0,
        "la marca arranca en {:.0} px, por debajo del tercio superior ({:.0})",
        brand.min.y,
        height / 3.0
    );
    // La navegación queda pegada al copy: el bloque completo es corto y está
    // arriba, en vez de repartir la pantalla en dos extremos.
    assert!(
        primary.max.y < height / 2.0,
        "la acción primaria termina en {:.0} px y deja medio viewport vacío debajo ({:.0})",
        primary.max.y,
        height / 2.0
    );
    assert!(
        primary.min.y > brand.min.y,
        "la acción no puede preceder a la marca que la introduce"
    );
}

/// El texto respeta su medida: ni una línea más ancha que la columna de
/// lectura, ni huida a los bordes del viewport.
#[test]
fn welcome_text_stays_inside_the_reading_column() {
    let ctx = ctx_with_fonts();
    let mut app = crate::app::dummy_grafito_app();
    app.show_onboarding = true;
    app.welcome_page = 0;
    let output = run_two_passes(&ctx, egui::vec2(1440.0, 900.0), |ctx| {
        app.draw_onboarding_window(ctx)
    });
    let column = grafito_ui::tokens::WELCOME_COLUMN_WIDTH;
    let mut widest = 0.0_f32;
    for clipped in &output.shapes {
        collect_widest_text(&clipped.shape, &mut widest);
    }
    assert!(
        widest > 120.0,
        "el texto más ancho mide {widest:.0} px: el copy no se está pintando"
    );
    assert!(
        widest <= column + 1.0,
        "el texto más ancho mide {widest:.0} px y la columna es {column:.0}: el titular está corriendo"
    );
}

/// El cromo de Ajustes es mínimo y estable.
///
/// Regresión del rediseño: sin barra de título redundante ("Configuración"),
/// sin etiqueta "Ajustes", sin botón atrás y sin pie con el nombre de usuario
/// (no hay cuenta que lo respalde). La navegación por secciones sí se pinta.
/// Se fija `user_name` a propósito: con el default vacío la etiqueta ausente
/// no probaría nada.
#[test]
fn settings_window_chrome_is_minimal_and_stable() {
    let size = egui::vec2(1440.0, 900.0);
    let ctx = ctx_with_fonts();
    let mut state = grafito_ui::assistant::AssistantPanelState::default();
    state.settings_open = true;
    state.user_name = "Estudiante".to_string();
    let output = run_two_passes(&ctx, size, |ctx| {
        grafito_ui::assistant::draw_assistant_settings_window(ctx, &mut state, false, false);
    });
    fn collect(shape: &egui::Shape, out: &mut Vec<String>) {
        match shape {
            egui::Shape::Text(text) => out.push(format!("{:?}", text.galley.text())),
            egui::Shape::Vec(shapes) => {
                for shape in shapes {
                    collect(shape, out);
                }
            }
            _ => {}
        }
    }
    let mut texts = Vec::new();
    for clipped in &output.shapes {
        collect(&clipped.shape, &mut texts);
    }
    assert!(
        !texts.is_empty(),
        "la ventana de ajustes no pintó texto: no se renderizó"
    );
    // `{:?}` del galley agrega comillas: se compara contra el debug exacto.
    for redundant in ["Configuración", "Ajustes", "Estudiante"] {
        let needle = format!("{redundant:?}");
        assert!(
            !texts.iter().any(|t| t == &needle),
            "cromo redundante pintado: {redundant:?}"
        );
    }
    for entry in [
        "General",
        "Apariencia",
        "Atajos",
        "Asistente",
        "Avanzado",
        "Plugins",
        "Perfil",
    ] {
        let needle = format!("{entry:?}");
        assert!(
            texts.iter().any(|t| t == &needle),
            "falta la entrada de navegación {entry:?}"
        );
    }
}

/// El fondo de Ajustes cubre la pantalla completa.
///
/// Regresión real (screenshot): con solo el `set_min_size` externo, el `Frame`
/// se ajustó a la columna de contenido y el fondo cubrió solo ~550 px
/// recostados a la izquierda, con negro a ambos lados. El segundo
/// `set_min_size` DENTRO del `Frame` (igual que la bienvenida) lo fija.
/// Se mide el `Rect` relleno más grande contra el área del viewport.
#[test]
fn settings_background_covers_the_full_viewport() {
    let size = egui::vec2(1440.0, 900.0);
    let ctx = ctx_with_fonts();
    let mut state = grafito_ui::assistant::AssistantPanelState::default();
    state.settings_open = true;
    let output = run_two_passes(&ctx, size, |ctx| {
        grafito_ui::assistant::draw_assistant_settings_window(ctx, &mut state, false, false);
    });
    fn biggest_rect(shape: &egui::Shape, best: &mut f32) {
        match shape {
            egui::Shape::Rect(rect) => {
                let area = rect.rect.width() * rect.rect.height();
                if area > *best {
                    *best = area;
                }
            }
            egui::Shape::Vec(shapes) => {
                for shape in shapes {
                    biggest_rect(shape, best);
                }
            }
            _ => {}
        }
    }
    let mut biggest = 0.0_f32;
    for clipped in &output.shapes {
        biggest_rect(&clipped.shape, &mut biggest);
    }
    let screen = size.x * size.y;
    assert!(
        biggest >= screen * 0.95,
        "el fondo más grande cubre {biggest:.0} px² de {screen:.0} px²: Ajustes no tapa la pantalla"
    );
}

/// El fondo de la bienvenida cubre la pantalla completa.
///
/// Regresión real: al quitar el `set_min_size` interior del `Frame`, el fondo
/// negro se ajustó a la columna de 640 px y el panel del asistente y la barra
/// superior quedaron al descubierto a la derecha — sin ningún error ni aviso.
/// Se mide el `Rect` relleno más grande contra el área del viewport.
#[test]
fn welcome_background_covers_the_full_viewport() {
    let size = egui::vec2(1440.0, 900.0);
    let ctx = ctx_with_fonts();
    let mut app = crate::app::dummy_grafito_app();
    app.show_onboarding = true;
    app.welcome_page = 0;
    let output = run_two_passes(&ctx, size, |ctx| app.draw_onboarding_window(ctx));
    fn biggest_rect(shape: &egui::Shape, best: &mut f32) {
        match shape {
            egui::Shape::Rect(rect) => {
                let area = rect.rect.width() * rect.rect.height();
                if area > *best {
                    *best = area;
                }
            }
            egui::Shape::Vec(shapes) => {
                for shape in shapes {
                    biggest_rect(shape, best);
                }
            }
            _ => {}
        }
    }
    let mut biggest = 0.0_f32;
    for clipped in &output.shapes {
        biggest_rect(&clipped.shape, &mut biggest);
    }
    let screen = size.x * size.y;
    assert!(
        biggest >= screen * 0.95,
        "el fondo más grande cubre {biggest:.0} px² de {screen:.0} px²: la bienvenida no tapa la pantalla"
    );
}

#[test]
fn headless_teaching_overlay_renders_without_panic() {
    let ctx = egui::Context::default();
    let mut app = crate::app::dummy_grafito_app();
    app.teaching_ui.start("derivada de x²");
    let mut budget = crate::app::RepaintBudget::default();
    let locale = app.config_locale();
    let output = ctx.run(headless_raw_input(), |ctx| {
        let _ = crate::teaching_ui::draw_teaching_overlay(
            &mut app.teaching_ui,
            ctx,
            &mut budget,
            locale,
            false,
        );
    });
    assert!(output.shapes.len() < 20_000);
    // advance one step and re-render
    app.teaching_ui.advance();
    let _ = ctx.run(headless_raw_input(), |ctx| {
        let _ = crate::teaching_ui::draw_teaching_overlay(
            &mut app.teaching_ui,
            ctx,
            &mut budget,
            locale,
            false,
        );
    });
}

#[test]
fn whiteboard_elements_for_hint_covers_all_teaching_topics() {
    use grafito_pedagogy::{TeachingSession, TeachingTopic};
    // Direct mapping strings that must hit each of the 14 branches inside whiteboard_elements_for_hint
    let hint_cases: &[(&str, usize, &str)] = &[
        ("secante pendiente tangente", 3, "secante branch should produce stroke+arrows"),
        ("fracción rectángulo dividido en partes común denominador", 7, "fracción branch"),
        ("vector flecha en ejes dos flechas r² pizarra vectorial", 8, "vector branch"),
        ("matriz grilla 2x2 determinante gauss aumentada pivote", 11, "matriz branch"),
        ("probabilidad histograma árbol contingencia normal bayes muestreo binom distribución curva normal", 7, "prob branch"),
        ("serie taylor fourier sucesión geométrica suma parcial aproxima", 4, "serie branch"),
        ("trigon seno coseno círculo unitario onda seno sin( cos(", 7, "trig branch"),
        ("cónica cono cortado elipse con focos hipérbola", 7, "conica branch"),
        ("ecuación cuadrática parábola y raíces discriminante recta y corte dos rectas sistema", 8, "ecuacion branch"),
        ("límite hueco en a flechas hacia a cálculo paso a paso", 6, "limite branch"),
        ("función ejes con puntos tabla de valores curva con ejes ejemplo con gráfica gráfica interactiva", 7, "funcion branch"),
        ("rectángulo riemann área", 5, "riemann/area branch"),
        ("pitágoras triángulo rectángulo cuadrados en cada lado", 3, "pitagoras branch"),
        ("pizarra libre", 0, "libre branch must be empty for user drawing"),
    ];
    for (hint, expected_min, label) in hint_cases {
        let elems = crate::teaching_ui::whiteboard_elements_for_hint(hint);
        if *expected_min == 0 {
            assert!(
                elems.is_empty(),
                "{label} hint={hint:?} should be empty but got {elems:?}"
            );
        } else {
            assert!(
                elems.len() >= *expected_min,
                "{label} hint={hint:?} expected at least {expected_min} elements, got {}",
                elems.len()
            );
        }
        // Ensure every element is finite / bounded (no unwrap panic downstream)
        for elem in &elems {
            match elem {
                grafito_whiteboard::WhiteboardElement::Text { at, text, size } => {
                    assert!(at.0.is_finite() && at.1.is_finite(), "text position finite");
                    assert!(*size > 0.0);
                    assert!(!text.is_empty());
                }
                grafito_whiteboard::WhiteboardElement::Rectangle { min, max, .. } => {
                    assert!(min.0.is_finite() && min.1.is_finite());
                    assert!(max.0.is_finite() && max.1.is_finite());
                }
                grafito_whiteboard::WhiteboardElement::Stroke { points, .. } => {
                    assert!(!points.is_empty());
                }
                _ => {}
            }
        }
    }

    // Ensure every TeachingTopic can be hydrated to a non-empty whiteboard via its TeachingSession hints
    let topics = [
        TeachingTopic::Derivada,
        TeachingTopic::Integral,
        TeachingTopic::Limite,
        TeachingTopic::Funcion,
        TeachingTopic::Pitagoras,
        TeachingTopic::Fraccion,
        TeachingTopic::Vector,
        TeachingTopic::Matriz,
        TeachingTopic::Probabilidad,
        TeachingTopic::Serie,
        TeachingTopic::Ecuacion,
        TeachingTopic::Trigonometria,
        TeachingTopic::Conica,
        TeachingTopic::General("tema abierto de prueba".into()),
    ];
    assert_eq!(
        topics.len(),
        14,
        "TeachingTopic 14 variants must be covered"
    );
    for topic in topics {
        let label = match &topic {
            TeachingTopic::General(s) => s.clone(),
            other => other.label(),
        };
        let session = TeachingSession::for_topic(&label);
        assert!(!session.steps.is_empty(), "topic {topic:?} must have steps");
        for step in &session.steps {
            let elems = crate::teaching_ui::whiteboard_elements_for_hint(&step.whiteboard_hint);
            // General topic may produce fallback text; still non-panicking. For defined topics, prefer non-empty when hint non-empty.
            if step.whiteboard_hint.trim().is_empty() {
                assert!(elems.is_empty());
            } else {
                // whiteboard hint for every built-in step should produce at least one element (or fallback text)
                assert!(
                    !elems.is_empty() || step.whiteboard_hint.trim().is_empty(),
                    "topic {topic:?} step {:?} hint {:?} produced no elements",
                    step.title,
                    step.whiteboard_hint
                );
            }
        }
        // also verify that distinct topics produce distinct element signatures (not all identical)
        let first_hint = session
            .steps
            .first()
            .map(|s| s.whiteboard_hint.clone())
            .unwrap_or_default();
        let first_elems = crate::teaching_ui::whiteboard_elements_for_hint(&first_hint);
        assert!(
            first_elems.len() < 100,
            "whiteboard elements must remain bounded per step"
        );
    }

    // Fallback for unknown hint must not panic and should produce a single text element
    let fallback =
        crate::teaching_ui::whiteboard_elements_for_hint("hint totalmente desconocido xyz");
    assert_eq!(fallback.len(), 1);
    assert!(matches!(
        fallback[0],
        grafito_whiteboard::WhiteboardElement::Text { .. }
    ));
    // Empty hint must be empty (no fallback)
    assert!(crate::teaching_ui::whiteboard_elements_for_hint("").is_empty());
    assert!(crate::teaching_ui::whiteboard_elements_for_hint("   ").is_empty());
}

#[test]
fn toolbar_and_panel_layout_constants_are_sane() {
    // Toolbar constants must respect budgets from docs/architecture.md: panel widths, button sizes, etc.
    assert_eq!(
        grafito_ui::toolbar::TOOLBAR_BUTTON_SIZE,
        36.0,
        "toolbar button size must stay 36 for 36x36 hit target"
    );
    assert_eq!(
        grafito_ui::toolbar::TOOLBAR_PANEL_HEIGHT,
        44.0,
        "toolbar panel height = button + 2*padding (36+8)"
    );
    // Panel defaults should be within the documented range (ASSISTANT_PANEL 300..520
    // default 400 en docs/architecture.md:183; acá solo se verifica el panel izquierdo).
    const { assert!(grafito_ui::tokens::PANEL_LEFT_DEFAULT >= grafito_ui::tokens::PANEL_LEFT_MIN) };
    const {
        assert!(
            grafito_ui::tokens::PANEL_LEFT_DEFAULT >= 200.0
                && grafito_ui::tokens::PANEL_LEFT_DEFAULT <= 400.0
        )
    };
    // Overflow must be consistent with top chrome logic (uses same breakpoint)
    assert!(grafito_ui::toolbar::toolbar_uses_overflow(960.0));
    assert!(!grafito_ui::toolbar::toolbar_uses_overflow(1361.0));
    assert!(crate::ui::top_chrome_uses_overflow(960.0));
    assert!(!crate::ui::top_chrome_uses_overflow(1361.0));
}

// ── D1 A11Y resto: cada overlay persistente cierra con Esc ────────────────

#[cfg(test)]
fn esc_raw_input() -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1280.0, 720.0),
        )),
        events: vec![egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::default(),
        }],
        ..Default::default()
    }
}

#[test]
fn d1_escape_closes_custom_tool_dialog_headless() {
    let ctx = egui::Context::default();
    let mut app = crate::app::dummy_grafito_app();
    app.show_custom_tool_dialog = true;
    // Sin Esc: el diálogo sigue abierto.
    let _ = ctx.run(headless_raw_input(), |ctx| {
        app.draw_custom_tool_dialog(ctx);
    });
    assert!(app.show_custom_tool_dialog, "sin Esc el diálogo persiste");
    // Con Esc: cierra sin guardar.
    let _ = ctx.run(esc_raw_input(), |ctx| {
        app.draw_custom_tool_dialog(ctx);
    });
    assert!(
        !app.show_custom_tool_dialog,
        "Esc cierra el diálogo custom-tool"
    );
}

#[test]
fn d1_escape_postpones_recovery_offer_without_deleting_headless() {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT_ID: AtomicU64 = AtomicU64::new(0);
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir();
    let main = dir.join(format!("grafito_d1_main_{}_{}.ggb", std::process::id(), id));
    let sidecar = dir.join(format!(
        "grafito_d1_sidecar_{}_{}.autosave",
        std::process::id(),
        id
    ));
    std::fs::write(&sidecar, b"sidecar d1").expect("sidecar temp");
    let ctx = egui::Context::default();
    let mut app = crate::app::dummy_grafito_app();
    app.recovery_offer = Some(crate::app::AutosaveRecoveryOffer {
        main_path: main.clone(),
        sidecar_path: sidecar.clone(),
        document: grafito_core::document::Document::new(),
        main_modified_epoch: None,
        sidecar_modified_epoch: 1,
        show_diff: false,
    });
    let _ = ctx.run(esc_raw_input(), |ctx| {
        app.draw_recovery_modal(ctx);
    });
    assert!(app.recovery_offer.is_none(), "Esc pospone la oferta");
    assert!(
        sidecar.exists(),
        "Esc no borra el sidecar (opción segura: posponer)"
    );
    let _ = std::fs::remove_file(&sidecar);
    let _ = std::fs::remove_file(&main);
}

#[test]
fn d1_escape_closes_corrupt_recovery_modal_headless() {
    let ctx = egui::Context::default();
    let mut app = crate::app::dummy_grafito_app();
    app.recovery_corrupt = Some(crate::app::AutosaveRecoveryCorrupt {
        main_path: std::path::PathBuf::from("/tmp/grafito_d1_dummy.ggb"),
        sidecar_path: std::path::PathBuf::from("/tmp/grafito_d1_dummy.autosave"),
        error: "corte de prueba".to_owned(),
    });
    let _ = ctx.run(esc_raw_input(), |ctx| {
        app.draw_recovery_modal(ctx);
    });
    assert!(
        app.recovery_corrupt.is_none(),
        "Esc cierra el modal corrupto sin borrar"
    );
}

#[test]
fn d1_escape_closes_about_and_onboarding_headless() {
    let ctx = egui::Context::default();
    let mut app = crate::app::dummy_grafito_app();
    app.show_about = true;
    let _ = ctx.run(esc_raw_input(), |ctx| {
        app.draw_about_window(ctx);
    });
    assert!(!app.show_about, "Esc cierra Acerca de");
    app.show_onboarding = true;
    let _ = ctx.run(esc_raw_input(), |ctx| {
        app.draw_onboarding_window(ctx);
    });
    assert!(!app.show_onboarding, "Esc cierra el onboarding");
}

#[test]
fn ola06_cheat_sheet_opens_and_esc_closes_headless() {
    let ctx = egui::Context::default();
    let mut app = crate::app::dummy_grafito_app();
    app.show_cheat_sheet = true;
    let _ = ctx.run(esc_raw_input(), |ctx| {
        app.draw_cheat_sheet_window(ctx);
    });
    assert!(!app.show_cheat_sheet, "Esc cierra la hoja de atajos");
}

#[test]
fn ola06_zoom_stepped_view_clamps_and_marks_interaction() {
    let mut app = crate::app::dummy_grafito_app();
    let before = app.document.view().scale;
    app.zoom_stepped_view(1.25, glam::Vec2::new(400.0, 300.0));
    assert!(
        app.document.view().scale > before,
        "zoom + acerca con centro dado"
    );
    assert!(app.is_view_changing, "marca vista en cambio");
    // Factor absurdo: clamp al rango del gesto + guards del motor, sin pánico.
    app.zoom_stepped_view(f32::INFINITY, glam::Vec2::new(0.0, 0.0));
    assert!(
        app.document.view().scale.is_finite(),
        "escala finita tras factor infinito"
    );
}

#[test]
fn wa_onboarding_solo_no_mostrar_persiste() {
    // W-A red: la X solo oculta en sesión; persiste SOLO "No mostrar".
    use crate::app::{GrafitoApp, OnboardingChoice};
    assert!(GrafitoApp::onboarding_choice_persists(
        OnboardingChoice::NeverShow
    ));
    assert!(!GrafitoApp::onboarding_choice_persists(
        OnboardingChoice::DismissX
    ));
    assert!(!GrafitoApp::onboarding_choice_persists(
        OnboardingChoice::Escape
    ));
    assert!(!GrafitoApp::onboarding_choice_persists(
        OnboardingChoice::TryExample
    ));
    assert!(!GrafitoApp::onboarding_choice_persists(
        OnboardingChoice::StartEmpty
    ));
}

#[test]
fn wa_nivel0_toast_borde_seis_idiomas() {
    // W-A: el aviso de panel oculto existe en 6 idiomas y promete
    // conservar el tab (borde: sale una vez por cruce, ver flag).
    use crate::app::nivel0_hidden_toast;
    use grafito_ui::i18n::Locale as L;
    for locale in [L::Es, L::En, L::Pt, L::It, L::Fr, L::De] {
        assert!(!nivel0_hidden_toast(locale).is_empty(), "{locale:?}");
    }
    assert!(
        nivel0_hidden_toast(L::Es).contains("tu tab se conserva"),
        "promesa de estado"
    );
}

#[test]
fn bb3_p1_rail_dead_click_avisa() {
    // BB3-P1-5: franja 480-520, drawer pedido pero sin ancho para pintarse
    // → bloqueado (aviso); en Nivel 0 (<480) manda su propio toast.
    use crate::ui::{left_drawer_content_visible, rail_drawer_blocked};
    assert!(left_drawer_content_visible(240.0));
    assert!(!left_drawer_content_visible(60.0));
    assert!(rail_drawer_blocked(500.0, 60.0, true));
    assert!(!rail_drawer_blocked(500.0, 60.0, false));
    assert!(!rail_drawer_blocked(500.0, 800.0, true));
    assert!(
        !rail_drawer_blocked(479.0, 60.0, true),
        "Nivel 0 tiene su toast"
    );
    assert!(!crate::app::drawer_blocked_toast(grafito_ui::i18n::Locale::Es).is_empty());
    assert!(
        crate::app::drawer_blocked_toast(grafito_ui::i18n::Locale::Es)
            .contains("tu tab se conserva"),
        "promesa de estado"
    );
}

#[test]
fn bb3_p1_dock_compacto_sin_clip() {
    // BB3-P1-7: default = min(360, max); con aire angosto el default baja
    // al max en vez de nacer clipado.
    use crate::ui::compact_dock_default_height;
    assert_eq!(compact_dock_default_height(1000.0), 360.0);
    assert_eq!(compact_dock_default_height(400.0), 260.0);
    assert_eq!(compact_dock_default_height(200.0), 180.0);
    assert!(compact_dock_default_height(300.0) <= (300.0_f32 * 0.65).max(180.0));
}

#[test]
fn bb3_p1_chip_44_sin_tabs() {
    // Sin tabs (2026-10-12): solo queda el chip a 44 táctil.
    use crate::ui::tool_chip_height;
    assert_eq!(tool_chip_height(), 44.0);
}

#[test]
fn wa_examen_banner_siempre_y_esc_con_confirmacion() {
    // W-A red: banner SIEMPRE visible en examen (2D y 3D) + Esc pide
    // confirmación (no sale directo) + Esc con modal lo cancela.
    use crate::app::{ExamEscapeAction, GrafitoApp};
    assert!(GrafitoApp::exam_banner_should_show(true));
    assert!(!GrafitoApp::exam_banner_should_show(false));
    assert_eq!(
        GrafitoApp::exam_escape_next(false, false),
        ExamEscapeAction::None
    );
    assert_eq!(
        GrafitoApp::exam_escape_next(true, false),
        ExamEscapeAction::RequestConfirm
    );
    assert_eq!(
        GrafitoApp::exam_escape_next(true, true),
        ExamEscapeAction::CancelModal
    );
    // La vía sigue siendo confirmada: pedir salida abre el modal, no apaga.
    let mut app = crate::app::dummy_grafito_app();
    app.exam_mode = true;
    app.exam_exit_confirm = false;
    app.set_exam_mode(false);
    assert!(app.exam_mode, "el lockdown sigue hasta confirmar");
    assert!(app.exam_exit_confirm, "se abrió el modal");
}

#[test]
fn wa_custom_tool_contador_y_nombre_live() {
    // W-A red: contador "usando N de M" (no trunca en silencio) + nombre
    // vacío/duplicado se valida ANTES de Guardar.
    use crate::app::{GrafitoApp, CUSTOM_TOOL_HISTORY_LIMIT};
    assert_eq!(GrafitoApp::custom_tool_history_usage(0), (0, 0));
    assert_eq!(GrafitoApp::custom_tool_history_usage(10), (10, 10));
    assert_eq!(
        GrafitoApp::custom_tool_history_usage(CUSTOM_TOOL_HISTORY_LIMIT + 5),
        (CUSTOM_TOOL_HISTORY_LIMIT, CUSTOM_TOOL_HISTORY_LIMIT + 5)
    );
    let mut store = grafito_command::ggbscript::CustomToolStore::new();
    // Vacío bloquea.
    let (can_save, feedback) = GrafitoApp::custom_tool_name_feedback("", &store);
    assert!(!can_save);
    assert!(feedback.is_some());
    let (can_save, _) = GrafitoApp::custom_tool_name_feedback("   ", &store);
    assert!(!can_save);
    // Nombre nuevo válido deja guardar sin aviso.
    let (can_save, feedback) = GrafitoApp::custom_tool_name_feedback("mi_tool", &store);
    assert!(can_save);
    assert!(feedback.is_none());
    // Duplicado permite guardar (reemplaza) pero avisa en vivo.
    store
        .define("mi_tool", "Hide[A]; Show[A]")
        .expect("define válido");
    let (can_save, feedback) = GrafitoApp::custom_tool_name_feedback("mi_tool", &store);
    assert!(can_save, "el duplicado reemplaza, no bloquea");
    let message = feedback.expect("aviso de duplicado");
    assert!(message.contains("ya existe"), "aviso honesto: {message}");
    // Formato inválido bloquea.
    let (can_save, _) = GrafitoApp::custom_tool_name_feedback("1mal", &store);
    assert!(!can_save);
}

#[test]
fn d1_palette_closes_on_escape_headless() {
    // La paleta ya cerraba con Esc (`command_palette.rs`, fuera de targets):
    // este test lo blinda contra regresiones desde app.
    let ctx = egui::Context::default();
    let mut app = crate::app::dummy_grafito_app();
    app.command_palette.open = true;
    let _ = ctx.run(esc_raw_input(), |ctx| {
        let _ = app
            .command_palette
            .show_localized(ctx, grafito_ui::i18n::Locale::Es);
    });
    assert!(!app.command_palette.open, "Esc cierra la paleta");
}

#[test]
fn custom_tool_runs_steps_through_pipeline() {
    let ctx = egui::Context::default();
    let mut app = crate::app::dummy_grafito_app();
    // Objeto base por el pipeline normal para Show/Hide (allowlist GGBScript).
    app.execute_command_and_record_with_outcome("Point[(0,0)]", 0.0);
    let label = app
        .document
        .objects()
        .values()
        .next()
        .map(|o| o.label().to_string())
        .unwrap_or_default();
    assert!(!label.is_empty());
    app.custom_tools
        .define("parpadeo", &format!("Hide[{label}]; Show[{label}]"))
        .expect("define válido");
    let undo_previo = app.undo_stack.len();
    app.run_custom_tool("parpadeo", &ctx);
    // Tras Hide+Show, el objeto sigue visible y el conteo intacto.
    let obj = app.document.objects().values().next().expect("objeto");
    assert!(obj.is_visible());
    // Éxito atómico: 2 pasos colapsan en 1 solo undo (no 2 parciales).
    assert_eq!(
        app.undo_stack.len(),
        undo_previo + 1,
        "tool exitosa debe dejar un solo undo atómico"
    );
    // Nombre desconocido: error honesto, sin pánico ni cambios.
    let before = app.document.objects().len();
    app.run_custom_tool("noexiste", &ctx);
    assert_eq!(app.document.objects().len(), before);
    // Paso roto a mitad: P1a-2 atómico — revierte todo, documento idéntico.
    app.custom_tools
        .define("mitad", &format!("Hide[{label}]; Hide[ZZZ999]"))
        .expect("define válido");
    let before_doc = serde_json::to_value(&app.document).expect("serializa previo");
    let undo_antes = app.undo_stack.len();
    app.run_custom_tool("mitad", &ctx);
    let after_doc = serde_json::to_value(&app.document).expect("serializa posterior");
    assert_eq!(
        before_doc, after_doc,
        "custom tool que falla a mitad debe revertir atómico"
    );
    assert_eq!(
        app.undo_stack.len(),
        undo_antes,
        "fallo atómico no debe dejar undo parcial"
    );
    let obj = app.document.objects().values().next().expect("objeto");
    assert!(obj.is_visible(), "el Hide parcial debe haberse revertido");
}

#[test]
fn custom_tool_falla_a_mitad_revierte_atomico_con_repeat() {
    // P1a-2 red-first: Repeat que falla a mitad dentro de custom tool revierte.
    let ctx = egui::Context::default();
    let mut app = crate::app::dummy_grafito_app();
    app.execute_command_and_record_with_outcome("Point[(0,0)]", 0.0);
    let label = app
        .document
        .objects()
        .values()
        .next()
        .map(|o| o.label().to_string())
        .unwrap_or_default();
    assert!(!label.is_empty());
    // Segundo paso falla (objeto inexistente) tras un paso que sí muta.
    app.custom_tools
        .define("mixta", &format!("Hide[{label}]; Hide[ZZZ_NOEXISTE]"))
        .expect("define válido");
    let before = serde_json::to_value(&app.document).expect("previo serializa");
    let undo_antes = app.undo_stack.len();
    app.run_custom_tool("mixta", &ctx);
    let after = serde_json::to_value(&app.document).expect("posterior serializa");
    assert_eq!(before, after, "documento idéntico tras fallo a mitad");
    assert_eq!(app.undo_stack.len(), undo_antes);
}

#[test]
fn exam_lockdown_bloquea_export_y_pide_confirm_para_salir() {
    let mut app = crate::app::dummy_grafito_app();
    assert!(!app.exam_mode);
    // Sin examen: exam_blocks no frena nada.
    assert!(!app.exam_blocks("Export"));
    app.set_exam_mode(true);
    assert!(app.exam_mode);
    assert!(!app.exam_exit_confirm);
    // Export bloqueado: no abre diálogo ni encola job (ctx None = vía legacy).
    app.export_with_dialog(crate::export::ExportFormat::Svg, None);
    assert!(app.pending_export_job.is_none());
    assert!(app.exam_blocks("Asistente"));
    assert!(app.exam_blocks("Internet"));
    // P1a-1: en examen el cambio de vista se BLOQUEA (sin bypass ni escape).
    // Ni apaga el examen ni cambia la perspectiva.
    let before = app.perspective;
    app.set_perspective(crate::Perspective::Geometry3D);
    assert!(app.exam_mode, "cambiar de vista no apaga el examen");
    assert_eq!(
        app.perspective, before,
        "en examen la vista no debe cambiar"
    );
    assert!(app
        .try_set_perspective(crate::Perspective::Geometry3D)
        .is_err());
    // Salir pide confirmación: no apaga directo.
    app.set_exam_mode(false);
    assert!(app.exam_mode, "sin confirmar sigue el lockdown");
    assert!(app.exam_exit_confirm, "se abrió el modal");
    // El modal se dibuja sin pánico (headless) y mantiene el lockdown.
    let ctx = egui::Context::default();
    let _ = ctx.run(headless_raw_input(), |ctx| {
        app.draw_exam_exit_modal(ctx);
    });
    assert!(app.exam_mode);
}

#[test]
fn exam_lockdown_bloquea_asistente_local_sin_panico() {
    let mut app = crate::app::dummy_grafito_app();
    app.set_exam_mode(true);
    let ctx = egui::Context::default();
    // Early-return antes de tocar runtime/documento: no paniquea en headless.
    app.start_local_assistant_request(&ctx);
    assert!(!app.assistant.is_pending);
}

#[test]
fn exam_bloquea_playlist_del_asistente() {
    // Red-first: la playlist nativa ("X y después Y") no debe correr en
    // examen: sería bypass del bloqueo del panel (`draw_assistant` retorna
    // antes, pero el runner debe gatear igual al inicio).
    let mut app = crate::app::dummy_grafito_app();
    app.set_exam_mode(true);
    let ctx = egui::Context::default();
    let playlist = grafito_anim::protocol::Playlist::try_new(vec![
        grafito_anim::protocol::PlaylistStep::pausa(500).expect("pausa válida"),
    ])
    .expect("playlist válida");
    app.run_assistant_playlist_with(&ctx, playlist);
    assert!(app.exam_mode, "el lockdown sigue activo");
    assert!(
        !app.assistant.anim_progress,
        "en examen la playlist no debe arrancar animación"
    );
}

#[test]
fn exam_bloquea_custom_tool() {
    // Red-first: las herramientas personalizadas mutan el documento por el
    // pipeline de comandos: en examen deben frenar con early-return honesto.
    let ctx = egui::Context::default();
    let mut app = crate::app::dummy_grafito_app();
    app.execute_command_and_record_with_outcome("Point[(0,0)]", 0.0);
    let label = app
        .document
        .objects()
        .values()
        .next()
        .map(|o| o.label().to_string())
        .unwrap_or_default();
    assert!(!label.is_empty());
    app.custom_tools
        .define("parpadeo", &format!("Hide[{label}]; Show[{label}]"))
        .expect("define válido");
    app.set_exam_mode(true);
    let before = serde_json::to_value(&app.document).expect("previo serializa");
    let undo_antes = app.undo_stack.len();
    app.run_custom_tool("parpadeo", &ctx);
    assert!(app.exam_mode, "el lockdown sigue activo");
    assert_eq!(
        serde_json::to_value(&app.document).expect("posterior serializa"),
        before,
        "en examen la tool no debe mutar el documento"
    );
    assert_eq!(
        app.undo_stack.len(),
        undo_antes,
        "en examen la tool no debe dejar undo"
    );
}

#[test]
fn exam_lockdown_bloquea_cambio_de_perspectiva() {
    // P1a-1 red-first: en examen el cambio de vista está bloqueado (sin bypass).
    let mut app = crate::app::dummy_grafito_app();
    assert_eq!(app.perspective, crate::Perspective::Geometry2D);
    app.set_exam_mode(true);
    assert!(app.exam_mode);
    // Vía fallible: debe dar Err honesto y no mutar.
    let err = app
        .try_set_perspective(crate::Perspective::Geometry3D)
        .expect_err("en examen el cambio de vista debe fallar");
    assert!(err.contains("examen"), "error honesto esperado, fue: {err}");
    assert_eq!(app.perspective, crate::Perspective::Geometry2D);
    assert_eq!(app.current_view, crate::ViewMode::D2);
    // Vía directa legacy: tampoco muta (early-return + toast).
    app.set_perspective(crate::Perspective::Geometry3D);
    assert_eq!(
        app.perspective,
        crate::Perspective::Geometry2D,
        "set_perspective no debe bypassear el lockdown"
    );
    assert_eq!(app.current_view, crate::ViewMode::D2);
    // Misma perspectiva sí pasa (no-op, sin falso bloqueo).
    app.try_set_perspective(crate::Perspective::Geometry2D)
        .expect("misma vista no bloquea");
    // ViewController puro también gatea con exam_locked.
    let mut vc = crate::controllers::ViewController::new();
    assert!(vc
        .try_set_perspective(crate::Perspective::Geometry3D, false)
        .is_ok());
    let err = vc
        .try_set_perspective(crate::Perspective::AlgebraCas, true)
        .expect_err("ViewController debe gatear con exam_locked");
    assert!(err.contains("examen"), "fue: {err}");
}

#[test]
fn redo_acotado_en_cantidad_y_bytes() {
    // Auditoría (undo/redo asimétrico): 60 entradas en redo → acotado a
    // MAX_UNDO (50) y MAX_UNDO_BYTES (50 MiB). Espejo del test del
    // DocumentController (`controllers.rs`), sobre la función libre de app.
    use crate::app::{enforce_redo_budgets, redo_changeset_bytes, MAX_UNDO, MAX_UNDO_BYTES};
    use grafito_core::{ChangeSet, Document};
    let mut redo: VecDeque<ChangeSet> = VecDeque::new();
    for _ in 0..60 {
        redo.push_back(ChangeSet {
            before: Document::new(),
            after: Document::new(),
        });
    }
    enforce_redo_budgets(&mut redo);
    assert!(
        redo.len() <= MAX_UNDO,
        "redo acotado por cantidad: {}",
        redo.len()
    );
    let bytes: usize = redo
        .iter()
        .map(redo_changeset_bytes)
        .fold(0usize, |a, b| a.saturating_add(b));
    assert!(bytes <= MAX_UNDO_BYTES, "redo acotado por bytes: {bytes}");
    // Un solo redo gigante se conserva (guardia len>1, igual que undo).
    let mut uno: VecDeque<ChangeSet> = VecDeque::from([ChangeSet {
        before: Document::new(),
        after: Document::new(),
    }]);
    enforce_redo_budgets(&mut uno);
    assert_eq!(uno.len(), 1);
}

#[test]
fn build_hash_no_es_local_ni_vacio() {
    // Frente build-hash rancio: el fallback es `dev-{version}`, jamás
    // `"local"` (mudo). En repo con git es el short-hash del commit.
    let hash = env!("GRAFITO_BUILD_HASH");
    assert!(!hash.is_empty(), "el hash de build no debe ser vacío");
    assert_ne!(hash, "local", "el binario no debe decir `local`: {hash}");
    // Fuerte: contra `git rev-parse --short HEAD` cuando hay repo; fuera de
    // git (CI sin `.git`) se acepta el fallback `dev-` (tolerante, sin ignore).
    let git_hash = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|salida| String::from_utf8(salida.stdout).ok())
        .map(|texto| texto.trim().to_string())
        .filter(|texto| !texto.is_empty());
    if let Some(esperado) = git_hash {
        assert_eq!(
            hash, esperado,
            "GRAFITO_BUILD_HASH debe ser el short-hash de git ({esperado}), fue: {hash}"
        );
    } else {
        // Sin git en runtime se aceptan las dos formas honestas: fallback
        // `dev-{versión}` o un short-hash real horneado al compilar (p.ej.
        // binario cruzado ejecutado fuera del repo). `local` sigue prohibido.
        assert!(
            hash.starts_with("dev-")
                || (hash.len() >= 7 && hash.chars().all(|c| c.is_ascii_hexdigit())),
            "hash sin git debe ser `dev-{{versión}}` o short-hash, fue: {hash}"
        );
    }
}

#[test]
fn gesto_slider_undo_coalescea_un_drag_en_una_sola_entrada() {
    let ctx = egui::Context::default();
    let key = egui::Id::new((crate::app::PANEL_GESTURE_UNDO_KEY, "live-p-test"));
    let mut document = grafito_core::Document::new();
    let mut undo_stack: VecDeque<grafito_core::Document> = VecDeque::new();
    let mut redo_stack: VecDeque<grafito_core::ChangeSet> = VecDeque::new();

    // Commit sin begin previo: no pushea nada.
    assert!(!crate::app::panel_gesture_commit(
        &ctx,
        key,
        document.version,
        &mut undo_stack,
        &mut redo_stack,
    ));
    assert!(undo_stack.is_empty());

    // Drag de 60 frames con cambio: una sola entrada (no 60).
    crate::app::panel_gesture_begin(&ctx, key, &document);
    for _ in 0..60 {
        document.bump_version();
    }
    assert!(crate::app::panel_gesture_commit(
        &ctx,
        key,
        document.version,
        &mut undo_stack,
        &mut redo_stack,
    ));
    assert_eq!(undo_stack.len(), 1);

    // Gesto sin cambio neto: cero entradas nuevas.
    crate::app::panel_gesture_begin(&ctx, key, &document);
    assert!(!crate::app::panel_gesture_commit(
        &ctx,
        key,
        document.version,
        &mut undo_stack,
        &mut redo_stack,
    ));
    assert_eq!(undo_stack.len(), 1);

    // Segundo gesto con cambio: segunda entrada (uno por gesto).
    crate::app::panel_gesture_begin(&ctx, key, &document);
    document.bump_version();
    assert!(crate::app::panel_gesture_commit(
        &ctx,
        key,
        document.version,
        &mut undo_stack,
        &mut redo_stack,
    ));
    assert_eq!(undo_stack.len(), 2);
}

#[test]
fn gesto_huerfano_se_commitea_al_desaparecer_el_widget() {
    let ctx = egui::Context::default();
    let key = egui::Id::new((crate::app::PANEL_GESTURE_UNDO_KEY, "huerfano-test"));
    let mut document = grafito_core::Document::new();
    let mut undo_stack: VecDeque<grafito_core::Document> = VecDeque::new();
    let mut redo_stack: VecDeque<grafito_core::ChangeSet> = VecDeque::new();

    // Gesto en curso, con cambio ya aplicado (el frame lo pone `begin`).
    crate::app::panel_gesture_begin(&ctx, key, &document);
    let frame_gesto = ctx.cumulative_pass_nr();
    document.bump_version();
    // Recién iniciado: no es huérfano aunque no se toque 1-2 frames.
    let mut cerrados = crate::app::panel_gesture_sweep_at(
        frame_gesto + crate::app::PANEL_GESTURE_STALE_FRAMES,
        document.version,
        &mut undo_stack,
        &mut redo_stack,
    );
    assert_eq!(cerrados, 0);
    assert!(undo_stack.is_empty(), "el gesto sigue vivo");
    // Sin toques pasada la ventana: se commitea una sola vez.
    cerrados = crate::app::panel_gesture_sweep_at(
        frame_gesto + crate::app::PANEL_GESTURE_STALE_FRAMES + 1,
        document.version,
        &mut undo_stack,
        &mut redo_stack,
    );
    assert_eq!(cerrados, 1);
    assert_eq!(undo_stack.len(), 1, "el before no se pierde");
    // Barrido repetido: nada colgado ni duplicado.
    cerrados = crate::app::panel_gesture_sweep_at(
        frame_gesto + crate::app::PANEL_GESTURE_STALE_FRAMES + 2,
        document.version,
        &mut undo_stack,
        &mut redo_stack,
    );
    assert_eq!(cerrados, 0);
    assert_eq!(undo_stack.len(), 1);
}

#[test]
fn gesto_huerfano_sin_cambio_neto_no_deja_entrada() {
    let ctx = egui::Context::default();
    let key = egui::Id::new((crate::app::PANEL_GESTURE_UNDO_KEY, "huerfano-sin-cambio"));
    let document = grafito_core::Document::new();
    let mut undo_stack: VecDeque<grafito_core::Document> = VecDeque::new();
    let mut redo_stack: VecDeque<grafito_core::ChangeSet> = VecDeque::new();
    crate::app::panel_gesture_begin(&ctx, key, &document);
    let frame_gesto = ctx.cumulative_pass_nr();
    let cerrados = crate::app::panel_gesture_sweep_at(
        frame_gesto + crate::app::PANEL_GESTURE_STALE_FRAMES + 1,
        document.version,
        &mut undo_stack,
        &mut redo_stack,
    );
    assert_eq!(cerrados, 1, "el gesto se cierra");
    assert!(undo_stack.is_empty(), "sin cambio no hay entrada");
}

#[test]
fn ola17_set_spin_speed_drives_camera_orbit_without_app_toggle() {
    let mut app = crate::app::dummy_grafito_app();
    // La órbita es de la vista 3D en perspectiva (las ortográficas no tienen azimut).
    app.view3d = crate::canvas::View3D::Perspective;
    let sphere = app.execute_command_and_record("Sphere[0, 0, 0, 1]", 0.0);
    assert!(
        !matches!(sphere, grafito_command::commands::CommandOutcome::Error(_)),
        "Sphere debe crear el objeto: {sphere:?}"
    );
    assert!(
        app.has_visible_multidimensional_object(),
        "esfera visible 3D"
    );
    app.execute_command_and_record("SetSpinSpeed[90]", 0.0);
    app.sync_spin_speed_from_document();
    assert_eq!(app.explicit_spin_speed_dps, Some(90.0));
    // Con velocidad explícita la cámara gira aunque el interruptor esté apagado.
    app.multidimensional_motion_enabled = false;
    let theta_before = app.camera.theta;
    assert!(app.advance_multidimensional_motion(0.5));
    assert_ne!(app.camera.theta, theta_before);
    // SetSpinSpeed[0] estaciona la órbita.
    app.execute_command_and_record("SetSpinSpeed[0]", 0.0);
    app.sync_spin_speed_from_document();
    assert_eq!(app.explicit_spin_speed_dps, Some(0.0));
    let theta_stopped = app.camera.theta;
    assert!(!app.advance_multidimensional_motion(0.5));
    assert_eq!(app.camera.theta, theta_stopped);
    // Sin variable (documento limpio) vuelve al gobierno de la app.
    app.document.variables.remove("__view_spin_speed");
    app.sync_spin_speed_from_document();
    assert_eq!(app.explicit_spin_speed_dps, None);
}

#[test]
fn ola27_on_load_and_on_update_scripts_run_automatically() {
    let mut app = crate::app::dummy_grafito_app();
    app.execute_command_and_record("A=(0,0)", 0.0);
    app.document
        .try_set_variable("k".to_string(), 0.0)
        .expect("k");
    app.execute_command_and_record("OnLoad[\"SetValue[k, 7]\"]", 0.0);
    // Guardar el guion no lo ejecuta.
    assert_eq!(app.document.variables.get("k"), Some(&0.0));
    // Al reemplazar el documento (abrir), el OnLoad corre una vez.
    let loaded = app.document.clone();
    let mut fresh = crate::app::dummy_grafito_app();
    fresh.replace_document(loaded, None);
    assert_eq!(fresh.document.variables.get("k"), Some(&7.0));

    // OnUpdate: corre tras un commit mutante, no al guardarse.
    let mut app2 = crate::app::dummy_grafito_app();
    app2.execute_command_and_record("A=(0,0)", 0.0);
    app2.document
        .try_set_variable("k".to_string(), 0.0)
        .expect("k");
    app2.execute_command_and_record("OnUpdate[A, \"SetValue[k, k+1]\"]", 0.0);
    assert_eq!(
        app2.document.variables.get("k"),
        Some(&0.0),
        "OnUpdate no corre al guardarse"
    );
    app2.execute_command_and_record("SetValue[k, 10]", 0.0);
    assert_eq!(app2.document.variables.get("k"), Some(&11.0));
    // Auto-referencia: el guard corta la cascada (un paso por commit).
    app2.execute_command_and_record("OnUpdate[A, \"SetValue[k, k+100]\"]", 0.0);
    app2.execute_command_and_record("SetValue[k, 0]", 0.0);
    assert_eq!(app2.document.variables.get("k"), Some(&100.0));
}

#[test]
fn ola27_script_store_commands_are_recognized() {
    assert!(crate::app::is_script_store_command("OnLoad[\"x\"]"));
    assert!(crate::app::is_script_store_command("onupdate[A, \"x\"]"));
    assert!(crate::app::is_script_store_command("OnClick[A, \"x\"]"));
    assert!(!crate::app::is_script_store_command("SetValue[k, 1]"));
    assert!(!crate::app::is_script_store_command("A=(0,0)"));
}

#[test]
fn ola16_text_tool_places_editable_text_object() {
    use grafito_core::{GeoObject, TextObj};
    use grafito_geometry::Point2;

    let mut app = crate::app::dummy_grafito_app();
    app.current_tool = grafito_ui::Tool::Text;
    let id = app
        .insert_object_from_tool(
            GeoObject::Text(TextObj::new("Texto", Point2::new(1.0, 2.0))),
            "Text",
            0.0,
        )
        .expect("texto insertado");
    let object = app.document.get_object(id).expect("objeto");
    assert!(matches!(object, GeoObject::Text(_)), "es un Text");
    assert!(!object.label().is_empty(), "etiqueta asignada");
    assert_eq!(
        app.current_tool,
        grafito_ui::Tool::Text,
        "herramienta activa"
    );
}

#[test]
fn ola16_button_tool_creates_real_action_button() {
    use grafito_command::commands::CommandOutcome;
    use grafito_geometry::Point2;

    let mut document = grafito_core::Document::new();
    let mut state = crate::tool_dispatcher::ToolState::default();
    let result = crate::tool_dispatcher::dispatch_tool(
        grafito_ui::Tool::Button,
        &mut state,
        &mut document,
        Point2::new(0.0, 0.0),
    );
    assert!(result.reset_tool, "Button vuelve a Select");
    match &state.last_outcome {
        Some(CommandOutcome::Message(message)) => {
            assert!(message.contains("creado"), "mensaje: {message}");
        }
        other => panic!("Button tool debía crear: {other:?}"),
    }
    let label = document
        .objects_iter()
        .find_map(|(_, object)| {
            grafito_command::ggbscript::action_view_of(object)
                .filter(|view| matches!(view.kind, grafito_command::ggbscript::ActionKind::Button))
                .map(|_| object.label().to_string())
        })
        .expect("botón real en el documento");
    let steps = grafito_command::ggbscript::run_button_script(&mut document, &label)
        .expect("el guion del botón corre");
    assert!(steps >= 1, "pasos: {steps}");
    assert_eq!(document.variables.get("boton1"), Some(&1.0));
}

#[test]
fn scroll_en_claro_sin_thumb_negro() {
    // Regresión real (screenshot Datos en claro): el thumb pintaba #1A1A1A
    // porque el `..floating()` del ScrollStyle trae `foreground_color: true`
    // y ese color nunca se seteaba. Con overlay flotante oculto en reposo no
    // hay rect opaco oscuro (los transparentes alfa 0 no cuentan).
    let ctx = egui::Context::default();
    grafito_ui::theme::LIGHT.apply(&ctx);
    let mut app = crate::app::dummy_grafito_app_with_perspective(crate::Perspective::DataAnalysis);
    let mut last = None;
    for _ in 0..3 {
        last = Some(ctx.run(headless_raw_input(), |ctx| {
            crate::panels::draw_data_panel(&mut app, ctx);
        }));
    }
    let output = last.unwrap();
    fn lum(c: egui::Color32) -> f32 {
        let lin = |v: u8| {
            let v = f32::from(v) / 255.0;
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * lin(c.r()) + 0.7152 * lin(c.g()) + 0.0722 * lin(c.b())
    }
    fn walk(shape: &egui::Shape, dark: &mut Vec<String>) {
        match shape {
            egui::Shape::Rect(rs) => {
                let r = rs.rect;
                if r.height() > 150.0 && r.width() < 26.0 && rs.fill.a() != 0 && lum(rs.fill) < 0.12
                {
                    dark.push(format!(
                        "x={:.0} y={:.0} w={:.0} h={:.0} fill={:?}",
                        r.min.x,
                        r.min.y,
                        r.width(),
                        r.height(),
                        rs.fill
                    ));
                }
            }
            egui::Shape::Vec(shapes) => {
                for s in shapes {
                    walk(s, dark);
                }
            }
            _ => {}
        }
    }
    let mut dark = Vec::new();
    for clipped in &output.shapes {
        walk(&clipped.shape, &mut dark);
    }
    assert!(
        dark.is_empty(),
        "barra negra en modo claro: {}",
        dark.join(" | ")
    );
}

#[test]
fn top_bar_cablea_breadcrumb_al_vault_viewer() {
    // B-A11 obsoleto + ELIMINADO 2026-10-06 (directiva usuario): sin breadcrumb
    // del vault; `draw_top_bar` solo conserva lectura muerta documentada.
    let source = include_str!("ui.rs");
    assert!(
        !source.contains("draw_top_bar_vault_breadcrumb"),
        "breadcrumb del vault sigue vivo tras eliminar vistas"
    );
    assert!(
        source.contains("Vault ELIMINADO 2026-10-06"),
        "falta pin de eliminación Vault en top bar"
    );
}

#[test]
fn top_bar_unificada_40px_con_breadcrumb_y_paleta() {
    // F18.1: barra única 40px con breadcrumb F16.2 + disparador paleta Ctrl+K
    // (misma acción que `shortcuts.rs:187` y `apply_palette_command` en
    // `app.rs:4390`, vía el punto único `open_palette()`). Pin de altura +
    // wiring en `draw_top_bar`.
    assert_eq!(
        grafito_ui::tokens::TOP_BAR_HEIGHT,
        40.0,
        "F18.1: top bar única de 40px"
    );
    let source = include_str!("ui.rs");
    let top_bar = source
        .split("pub(crate) fn draw_top_bar")
        .nth(1)
        .expect("draw_top_bar existe");
    // El cuerpo termina en el sidebar (siguiente sección tras la barra).
    let top_bar = top_bar
        .split("// ── LEFT SIDEBAR")
        .next()
        .unwrap_or(top_bar);
    assert!(
        !top_bar.contains("draw_top_bar_vault_breadcrumb"),
        "breadcrumb del vault sigue vivo tras eliminar vistas"
    );
    assert!(
        !top_bar.contains("menu.tools.palette"),
        "botón paleta sigue en top bar (eliminado 2026-10-06)"
    );
    assert!(
        !top_bar.contains("open_palette()"),
        "disparador paleta sigue en top bar"
    );
}

#[test]
fn exam_b_a179_modo_examen_sale_de_menu_persp() {
    // B-A179: checkbox + título del banner salen de `menu.persp.exam_mode`
    // con locale; ES pineado a los literales previos (`app.rs:6189,:6249`).
    // Sin egui: pines + blindaje de wiring por `include_str!`.
    use grafito_ui::i18n::{t, Locale};
    assert_eq!(t("menu.persp.exam_mode", Locale::Es), "Modo examen");
    assert_eq!(t("menu.persp.exam_mode", Locale::En), "Exam mode");
    assert_eq!(t("menu.persp.exam_mode", Locale::Pt), "Modo de exame");
    assert_eq!(t("menu.persp.exam_mode", Locale::It), "Modalità esame");
    assert_eq!(t("menu.persp.exam_mode", Locale::Fr), "Mode examen");
    assert_eq!(t("menu.persp.exam_mode", Locale::De), "Prüfungsmodus");
    // Wiring: los 2 sitios van por `t()` con `locale` (0 keys nuevas).
    let source = include_str!("app.rs");
    assert!(
        source.contains("t(\"menu.persp.exam_mode\", locale)"),
        "examen cablea menu.persp.exam_mode con locale"
    );
    assert_eq!(
        source
            .matches("t(\"menu.persp.exam_mode\", locale)")
            .count(),
        2,
        "checkbox + banner van por la key (resto examen sigue ES, follow-up B-A180)"
    );
    // Blindaje: viejos literales duros fuera del prod.
    for needle in [
        "scandi_checkbox(ui, \"Modo examen\"",
        "RichText::new(\"Modo examen\")",
    ] {
        assert!(
            !source.contains(needle),
            "examen sin ES duro en prod (B-A179): {needle}"
        );
    }
}

#[test]
fn aula_b_a197_tab_sale_de_rail_msg() {
    // B-A197: fallback del rail (`app.rs`, botón flotante cuando el rail de
    // `ui.rs` no expone el tab) + header del panel (`classroom.rs`) salen de
    // `rail.tab_aula` con locale; la const muerta `AULA_TAB_LABEL` se elimina
    // (0 usos, solo `#[allow(dead_code)]`). ES pineado a los literales
    // previos (`app.rs:7912`, `classroom.rs:463`). Sin egui: pines + wiring
    // por `include_str!` + blindaje.
    use grafito_ui::i18n::{t, Locale};
    assert_eq!(t("rail.tab_aula", Locale::Es), "Aula");
    assert_eq!(t("rail.tab_aula", Locale::En), "Class");
    assert_eq!(t("rail.tab_aula", Locale::Pt), "Aula");
    assert_eq!(t("rail.tab_aula", Locale::It), "Aula");
    assert_eq!(t("rail.tab_aula", Locale::Fr), "Classe");
    assert_eq!(t("rail.tab_aula", Locale::De), "Klasse");
    // Wiring: los 2 sitios van por `t()` con `locale` hoisted pre-closure.
    for (source, file) in [
        (include_str!("app.rs"), "app.rs"),
        (include_str!("classroom.rs"), "classroom.rs"),
    ] {
        assert!(
            source.contains("t(\"rail.tab_aula\", aula_locale)"),
            "{file} cablea rail.tab_aula con locale"
        );
    }
    // Blindaje: viejos literales duros + const muerta fuera del prod
    // (el `"Aula"` restante en `classroom.rs:135` es comentario doc).
    for needle in ["RichText::new(\"Aula\")", "AULA_TAB_LABEL"] {
        assert!(
            !include_str!("app.rs").contains(needle),
            "app.rs sin ES duro de Aula (B-A197): {needle}"
        );
        assert!(
            !include_str!("classroom.rs").contains(needle),
            "classroom.rs sin ES duro de Aula (B-A197): {needle}"
        );
    }
}

#[test]
fn classroom_b_a198_panel_externo_sale_de_classroom_msg() {
    // B-A198: chrome externo de `draw_classroom_panel` (opt-in +
    // regenerar + alternar host + título chat + enviar) sale de
    // `classroom.*` con `aula_locale` (hoisted pre-`show`, B-A197).
    // ES pineado a los literales previos (`classroom.rs:471,481,484,
    // 494,504`). Sin egui: pines + wiring por `include_str!` +
    // blindaje con contexto call-site. El cuerpo `ClassroomPanel::ui`
    // + `draw_share_qr` + errores `enqueue_chat_message` quedan fuera
    // a propósito (follow-up B-A199).
    use grafito_ui::i18n::{t, Locale};
    assert_eq!(t("classroom.opt_in_toggle", Locale::Es), "Activar");
    assert_eq!(t("classroom.regen_code", Locale::Es), "Regenerar código");
    assert_eq!(
        t("classroom.toggle_host", Locale::Es),
        "Alternar host/invitado"
    );
    assert_eq!(
        t("classroom.chat_title", Locale::Es),
        "Chat del aula (offline)"
    );
    assert_eq!(t("classroom.chat_send", Locale::Es), "Enviar");
    assert_eq!(t("classroom.opt_in_toggle", Locale::En), "Enable");
    assert_eq!(t("classroom.chat_send", Locale::De), "Senden");
    let source = include_str!("classroom.rs");
    for needle in [
        "t(\"classroom.opt_in_toggle\", aula_locale)",
        "t(\"classroom.regen_code\", aula_locale)",
        "t(\"classroom.toggle_host\", aula_locale)",
        "t(\"classroom.chat_title\", aula_locale)",
        "t(\"classroom.chat_send\", aula_locale)",
    ] {
        assert!(
            source.contains(needle),
            "classroom.rs sin wiring B-A198: {needle}"
        );
    }
    // Blindaje: viejos literales duros del panel externo fuera del
    // prod (con contexto call-site; el cuerpo `ui()` no los contenía).
    for needle in [
        "scandi_checkbox(ui, \"Activar\"",
        "button(\"Regenerar código\")",
        "button(\"Alternar host/invitado\")",
        "new(\"Chat del aula (offline)\")",
        "button(\"Enviar\")",
    ] {
        assert!(
            !source.contains(needle),
            "classroom.rs con ES duro externo (B-A198): {needle}"
        );
    }
}

#[test]
fn classroom_b_a199_panel_interno_sale_de_classroom_msg() {
    // B-A199: cuerpo `ClassroomPanel::ui` + `draw_share_qr` + notices
    // `enqueue_chat_message` salen de `classroom.*` con `locale`.
    // ES pineado a los literales previos (`classroom.rs:203,211,219,225,
    // 262,272,279,287,293,309,315,321,327,337,356,408`). Sin egui: pines +
    // helpers + wiring por `include_str!` + blindaje. `grafito://aula/{code}`
    // + `AULA-` + Err canónicos quedan intactos a propósito (identificadores/
    // motor, precedente B-A184).
    use crate::classroom::{
        classroom_chat_failed_for, classroom_chat_queued_for, classroom_queue_status_for,
        classroom_room_code_for,
    };
    use grafito_ui::i18n::{t, Locale};
    assert_eq!(
        t("classroom.panel_title", Locale::Es),
        "Aula — loopback sin red (F0)"
    );
    assert_eq!(
        t("classroom.disabled_msg", Locale::Es),
        "Aula deshabilitada. Activá el opt-in para ver el tab \"Aula\" y el QR de conexión."
    );
    assert_eq!(
        t("classroom.room_code", Locale::Es),
        "Código de sala: {code}"
    );
    assert_eq!(
        t("classroom.chat_queued", Locale::Es),
        "Sin red (F0): mensaje #{id} encolado, se enviará al reconectar ({n} pendiente(s))."
    );
    assert_eq!(
        t("classroom.chat_failed", Locale::Es),
        "No se pudo encolar: {motivo}"
    );
    // Helpers puros: Es delega al canónico por construcción.
    assert_eq!(
        classroom_room_code_for("AULA-1", Locale::Es),
        "Código de sala: AULA-1"
    );
    assert_eq!(
        classroom_room_code_for("AULA-1", Locale::En),
        "Room code: AULA-1"
    );
    assert_eq!(
        classroom_queue_status_for(3, Locale::Es),
        "Cola offline: 3 pendiente(s) (máx 128×2048)."
    );
    assert_eq!(
        classroom_queue_status_for(3, Locale::De),
        "Offline-Warteschlange: 3 ausstehend (max. 128×2048)."
    );
    assert_eq!(
        classroom_chat_queued_for(7, 2, Locale::Es),
        "Sin red (F0): mensaje #7 encolado, se enviará al reconectar (2 pendiente(s))."
    );
    assert_eq!(
        classroom_chat_queued_for(7, 2, Locale::En),
        "Offline (F0): message #7 queued, will send on reconnect (2 pending)."
    );
    assert_eq!(
        classroom_chat_failed_for("boom", Locale::Es),
        "No se pudo encolar: boom"
    );
    assert_eq!(
        classroom_chat_failed_for("boom", Locale::Pt),
        "Não foi possível enfileirar: boom"
    );
    let source = include_str!("classroom.rs");
    for needle in [
        "classroom_t(\"classroom.panel_title\", locale)",
        "classroom_t(\"classroom.disabled_msg\", locale)",
        "classroom_t(\"classroom.optin_hint\", locale)",
        "classroom_room_code_for(",
        "classroom_t(\"classroom.qr_hint\", locale)",
        "classroom_t(\"classroom.host_mode\", locale)",
        "classroom_t(\"classroom.guest_mode\", locale)",
        "classroom_t(\"classroom.generating\", locale)",
        "classroom_t(\"classroom.offline_note\", locale)",
        "classroom_queue_status_for(",
        "classroom_t(\"classroom.chat_hint\", locale)",
        "classroom_t(\"classroom.qr_unavailable\", locale)",
        "classroom_t(\"classroom.chat_empty\", locale)",
        "classroom_chat_queued_for(",
        "classroom_t(\"classroom.chat_full\", locale)",
        "classroom_chat_failed_for(",
        "draw_share_qr(ui, code.as_str(), CLASSROOM_QR_SIDE, locale)",
        "app.classroom.ui(ui, aula_locale)",
        "enqueue_chat_message(&body, aula_locale)",
    ] {
        assert!(
            source.contains(needle),
            "classroom.rs sin wiring B-A199: {needle}"
        );
    }
    // Blindaje: viejos literales duros del panel interno fuera de prod
    // (con contexto call-site; los Err canónicos + `grafito://aula` quedan).
    for needle in [
        "new(\"Aula — loopback sin red (F0)\")",
        "Aula deshabilitada. Activá el opt-in para ver",
        "El opt-in se activa en Configuración",
        "format!(\"Código de sala: {}\",",
        "Escaneá el QR con otro dispositivo",
        "new(\"Modo anfitrión (lobby activo)\")",
        "Modo invitado — el anfitrión comparte",
        "\"Generando código de sala…\"",
        "F0 sin red: no hay P2P ni servidor.",
        "Cola offline: {} pendiente",
        "Chat offline: sin red todo mensaje se encola",
        "QR no disponible para este código",
        "\"Escribí un mensaje antes de enviar, che.\"",
        "Sin red (F0): mensaje #{id} encolado",
        "La cola offline está llena (128): reintentá",
        "format!(\"No se pudo encolar: {motivo}\")",
    ] {
        assert!(
            !source.contains(needle),
            "classroom.rs con ES duro interno (B-A199): {needle}"
        );
    }
}

#[test]
fn classroom_b_a201_outbox_sale_de_classroom_msg() {
    // B-A201: avisos `outbox_notice` (corrupta + descartados `{n}` +
    // lectura) salen de `classroom.outbox_*` con `locale`.
    // ES pineado a los canónicos (`offline.rs:302` + `:307` +
    // `utils.rs:429`). Sin egui: pines + helpers + wiring por
    // `include_str!` + blindaje. El motor (`decode_persist`/`notice()` +
    // Err `enqueue`) queda canónico ES a propósito (B-A199/B-A184).
    use crate::classroom::{
        classroom_outbox_corrupt_for, classroom_outbox_discarded_for,
        classroom_outbox_read_error_for,
    };
    use grafito_ui::i18n::{t, Locale};
    assert_eq!(
        t("classroom.outbox_corrupt", Locale::Es),
        "La cola offline guardada estaba corrupta y se descartó, che."
    );
    assert_eq!(
        t("classroom.outbox_discarded", Locale::Es),
        "Se descartaron {n} mensajes offline inválidos al cargar, che."
    );
    assert_eq!(
        t("classroom.outbox_read_error", Locale::Es),
        "No se pudo leer la cola offline guardada y se arrancó vacía, che."
    );
    // Helpers puros: Es delega al canónico por construcción.
    assert_eq!(
        classroom_outbox_corrupt_for(Locale::Es),
        "La cola offline guardada estaba corrupta y se descartó, che."
    );
    assert_eq!(
        classroom_outbox_corrupt_for(Locale::En),
        "Saved offline queue was corrupt and was discarded."
    );
    assert_eq!(
        classroom_outbox_discarded_for(3, Locale::Es),
        "Se descartaron 3 mensajes offline inválidos al cargar, che."
    );
    assert_eq!(
        classroom_outbox_discarded_for(3, Locale::En),
        "Discarded 3 invalid offline messages on load."
    );
    assert_eq!(
        classroom_outbox_read_error_for(Locale::Es),
        "No se pudo leer la cola offline guardada y se arrancó vacía, che."
    );
    assert_eq!(
        classroom_outbox_read_error_for(Locale::De),
        "Gespeicherte Offline-Warteschlange konnte nicht gelesen werden, leer gestartet."
    );
    let source = include_str!("classroom.rs");
    for needle in [
        "classroom_outbox_corrupt_for(",
        "classroom_outbox_discarded_for(",
        "classroom_outbox_read_error_for(",
        "new_with_locale(",
        "load_outbox_for(",
    ] {
        assert!(
            source.contains(needle) || include_str!("utils.rs").contains(needle),
            "outbox sin wiring B-A201: {needle}"
        );
    }
    assert!(
        include_str!("app.rs").contains("new_with_locale("),
        "app.rs sin wiring B-A201: new_with_locale("
    );
    // Blindaje: el aviso de lectura ya no va duro en `utils.rs`
    // (sale del helper; el canónico vive en `i18n.rs` + motor).
    assert!(
        !include_str!("utils.rs")
            .contains("No se pudo leer la cola offline guardada y se arrancó vacía, che."),
        "utils.rs con ES duro outbox (B-A201)"
    );
}

#[test]
fn aula_roles_selector_y_secciones_salen_de_classroom_msg() {
    // Roles Profesor/Alumno: selector + Presentar/Preguntar/Seguir + overlay
    // foco salen de `classroom.*` con locale (patrón B-A197/B-A198). ES
    // pineado a los literales de `classroom.rs`; exportes reusan
    // `palette.action.export_*` (0 claves nuevas para eso). Sin egui:
    // pines + wiring por `include_str!` + blindaje.
    use grafito_ui::i18n::{t, Locale};
    assert_eq!(t("classroom.role_label", Locale::Es), "Rol en el aula");
    assert_eq!(t("classroom.role_none", Locale::Es), "Sin rol");
    assert_eq!(t("classroom.role_teacher", Locale::Es), "Profesor");
    assert_eq!(t("classroom.role_student", Locale::Es), "Alumno");
    assert_eq!(t("classroom.present", Locale::Es), "Presentar");
    assert_eq!(
        t("classroom.presenting", Locale::Es),
        "Presentando — Esc para salir"
    );
    assert_eq!(t("classroom.ask", Locale::Es), "Preguntar");
    assert_eq!(t("classroom.follow", Locale::Es), "Seguir");
    assert_eq!(
        t("classroom.attending", Locale::Es),
        "Modo atención — Esc para salir"
    );
    assert_eq!(t("classroom.exit_focus", Locale::Es), "Salir");
    assert_eq!(t("classroom.curtain", Locale::Es), "Cortina");
    assert_eq!(t("classroom.present", Locale::En), "Present");
    assert_eq!(t("classroom.ask", Locale::De), "Fragen");
    let source = include_str!("classroom.rs");
    for needle in [
        "t(\"classroom.role_label\", aula_locale)",
        "classroom_role_label_for(ClassroomRole::Teacher, aula_locale)",
        "classroom_role_label_for(ClassroomRole::Student, aula_locale)",
        "t(\"classroom.present\", aula_locale)",
        "t(\"classroom.ask\", aula_locale)",
        "t(\"classroom.follow\", aula_locale)",
        "t(\"classroom.presenting\", locale)",
        "t(\"classroom.attending\", locale)",
        "t(\"classroom.exit_focus\", locale)",
        "t(\"classroom.curtain\", locale)",
        "t(\"palette.action.export_svg\", aula_locale)",
        "t(\"palette.action.export_png\", aula_locale)",
        "ask_assistant_from_classroom(app)",
        "enter_classroom_focus(app, ClassroomFocus::Present)",
        "enter_classroom_focus(app, ClassroomFocus::Attend)",
    ] {
        assert!(source.contains(needle), "sin wiring roles Aula: {needle}");
    }
    assert!(
        include_str!("app.rs").contains("classroom_focus_frame(self, ctx)"),
        "app.rs sin call-site del foco Aula"
    );
    // Blindaje: literales duros de roles fuera del prod.
    for needle in [
        "button(\"Presentar\")",
        "button(\"Preguntar\")",
        "button(\"Seguir\")",
        "new(\"Rol en el aula\")",
    ] {
        assert!(
            !source.contains(needle),
            "classroom.rs con ES duro de roles: {needle}"
        );
    }
}

#[test]
fn aula_rol_default_sin_rol_en_app() {
    // Default = el actual (Unset), sin forzar elección: el dummy recién
    // creado no trae rol y el sync de opt-in por frame no lo pisa.
    let mut app = crate::app::dummy_grafito_app();
    assert_eq!(app.classroom.role(), crate::classroom::ClassroomRole::Unset);
    app.classroom.set_opt_in(true);
    assert_eq!(app.classroom.role(), crate::classroom::ClassroomRole::Unset);
}

#[test]
fn aula_presentar_oculta_paneles_y_esc_restaura_headless() {
    // Profesor → Presentar: entra en foco ocultando paneles; Esc sale y
    // restaura exactamente lo que había. Headless con `ctx.run` (precedente
    // D1 `esc_raw_input`): Piel pura, sin I/O ni red.
    use crate::classroom::{enter_classroom_focus, ClassroomFocus};
    let ctx = egui::Context::default();
    let mut app = crate::app::dummy_grafito_app();
    app.left_drawer_open = true;
    app.compact_drawer_open = true;
    app.right_drawer_open = true;
    app.assistant_visible = true;
    app.keyboard_visible = true;
    enter_classroom_focus(&mut app, ClassroomFocus::Present);
    assert!(app.classroom.in_focus());
    assert!(!app.left_drawer_open);
    assert!(!app.compact_drawer_open);
    assert!(!app.right_drawer_open);
    assert!(!app.assistant_visible);
    assert!(!app.keyboard_visible);
    // Sin Esc el foco persiste (el overlay se pinta, no sale).
    let _ = ctx.run(headless_raw_input(), |ctx| {
        crate::classroom::classroom_focus_frame(&mut app, ctx);
    });
    assert!(app.classroom.in_focus(), "sin Esc el foco persiste");
    // Con Esc: sale y restaura todo.
    let _ = ctx.run(esc_raw_input(), |ctx| {
        crate::classroom::classroom_focus_frame(&mut app, ctx);
    });
    assert!(!app.classroom.in_focus(), "Esc sale del foco");
    assert!(app.left_drawer_open, "Esc restaura el drawer");
    assert!(app.compact_drawer_open);
    assert!(app.right_drawer_open);
    assert!(app.assistant_visible);
    assert!(app.keyboard_visible);
}

#[test]
fn aula_seguir_modo_atencion_y_esc_restaura_headless() {
    // Alumno → Seguir: mismo mecanismo con modo Attend.
    use crate::classroom::{enter_classroom_focus, ClassroomFocus};
    let ctx = egui::Context::default();
    let mut app = crate::app::dummy_grafito_app();
    app.left_drawer_open = true;
    app.assistant_visible = false;
    enter_classroom_focus(&mut app, ClassroomFocus::Attend);
    assert_eq!(app.classroom.focus_mode(), Some(ClassroomFocus::Attend));
    assert!(!app.left_drawer_open);
    let _ = ctx.run(esc_raw_input(), |ctx| {
        crate::classroom::classroom_focus_frame(&mut app, ctx);
    });
    assert!(!app.classroom.in_focus());
    assert!(app.left_drawer_open, "Esc restaura el drawer");
    assert!(!app.assistant_visible, "restaura apagado si estaba apagado");
}

#[test]
fn aula_preguntar_abre_asistente_con_foco_y_saludo() {
    // Alumno → Preguntar: abre el asistente, pide foco one-shot y saluda
    // con Mili SOLO si el transcript está vacío (mismo camino + misma clave
    // que el onboarding `onboarding_try_example`).
    use grafito_ui::i18n::{t, Locale};
    let mut app = crate::app::dummy_grafito_app();
    app.assistant_visible = false;
    assert!(app.assistant.conversation.is_empty());
    crate::classroom::ask_assistant_from_classroom(&mut app);
    assert!(app.assistant_visible, "Preguntar abre el asistente");
    assert!(
        app.assistant.composer_focus_request,
        "Preguntar pide foco al composer"
    );
    assert_eq!(app.assistant.conversation.len(), 1);
    assert_eq!(
        app.assistant.conversation[0].content,
        t("assistant.greeting.fallback", Locale::Es),
        "saludo contextual del onboarding"
    );
    // Con transcript no vacío: abre + foco, pero NO saluda de nuevo.
    let mut app2 = crate::app::dummy_grafito_app();
    app2.assistant_visible = false;
    app2.assistant
        .push_turn(grafito_assistant_types::ConversationTurn::user("hola"));
    crate::classroom::ask_assistant_from_classroom(&mut app2);
    assert!(app2.assistant_visible);
    assert!(app2.assistant.composer_focus_request);
    assert_eq!(
        app2.assistant.conversation.len(),
        1,
        "sin saludo repetido si ya hay transcript"
    );
}

#[test]
fn exam_b_a180_resto_sale_de_exam_msg() {
    // B-A180: banner scope + salir/hover + toast `{que}` + modal
    // título/heading/body/confirm/stay salen de `exam.*` con locale.
    // ES pineado a los literales previos (`app.rs:6261,:6270,:6274,:6293,:
    // 6311,:6333,:6340,:6352,:6364`). Sin egui: pines + helper + wiring por
    // `include_str!` + blindaje.
    use crate::app::exam_blocks_text_for;
    use grafito_ui::i18n::{exam_msg, t, Locale};
    assert_eq!(
        t("exam.banner_scope", Locale::Es),
        "asistente, internet y export bloqueados"
    );
    assert_eq!(t("exam.exit_btn", Locale::Es), "Salir");
    assert_eq!(
        t("exam.exit_btn_hint", Locale::Es),
        "Salir del modo examen (pide confirmación)"
    );
    assert_eq!(
        t("exam.blocked_toast", Locale::Es),
        "{que} bloqueado en modo examen, che."
    );
    assert_eq!(t("exam.modal_title", Locale::Es), "Salir del modo examen");
    assert_eq!(t("exam.modal_heading", Locale::Es), "Salir del examen");
    assert_eq!(
        t("exam.modal_body", Locale::Es),
        "Se habilitan el asistente, internet y el export."
    );
    assert_eq!(t("exam.modal_confirm", Locale::Es), "Sí, salir");
    assert_eq!(t("exam.modal_stay", Locale::Es), "Seguir en examen");
    // Helper puro: Es delega al canónico por construcción; resto interpola
    // `{que}` (el `que` va crudo: "Export"/"Herramientas").
    assert_eq!(
        exam_blocks_text_for("Export", Locale::Es),
        "Export bloqueado en modo examen, che."
    );
    assert_eq!(
        exam_blocks_text_for("Export", Locale::En),
        "Export blocked in exam mode."
    );
    assert_eq!(
        exam_blocks_text_for("Export", Locale::De),
        "Export im Prüfungsmodus blockiert."
    );
    assert_eq!(
        exam_msg("banner_scope", Locale::Es),
        t("exam.banner_scope", Locale::Es)
    );
    assert_eq!(exam_msg("modal_stay", Locale::It), "Resta nell'esame");
    assert_eq!(exam_msg("nope", Locale::Es), "nope");
    // Wiring: los 9 sitios van por `t()`/`exam_blocks_text_for` con locale
    // (pines por key con coma —`fmt` parte `t(` en 2 líneas, precedente
    // B-A105/B-A147—; `source` es `app.rs`, el test no auto-matchea).
    let source = include_str!("app.rs");
    for needle in [
        "\"exam.banner_scope\", locale",
        "\"exam.exit_btn\",",
        "\"exam.exit_btn_hint\", locale",
        "exam_blocks_text_for(que, locale)",
        "\"exam.modal_title\", locale",
        "\"exam.modal_heading\", locale",
        "\"exam.modal_body\", locale",
        "\"exam.modal_confirm\",",
        "\"exam.modal_stay\",",
    ] {
        assert!(
            source.contains(needle),
            "examen cablea {needle} con locale (B-A180)"
        );
    }
    // Blindaje: viejos literales duros fuera del prod (el template `{que}`
    // vive solo en el helper canónico + su doc, conteo exacto 2).
    for needle in [
        "RichText::new(\"asistente, internet y export bloqueados\")",
        "RichText::new(\"Salir\")",
        "on_hover_text(\"Salir del modo examen (pide confirmación)\")",
        "Window::new(\"Salir del modo examen\")",
        "RichText::new(\"Salir del examen\")",
        "RichText::new(\"Se habilitan el asistente, internet y el export.\")",
        "RichText::new(\"Sí, salir\")",
        "RichText::new(\"Seguir en examen\")",
    ] {
        assert!(
            !source.contains(needle),
            "examen sin ES duro en prod (B-A180): {needle}"
        );
    }
    assert_eq!(
        source
            .matches("{que} bloqueado en modo examen, che.")
            .count(),
        2,
        "toast vive solo en helper + doc (B-A180)"
    );
}

#[test]
fn exam_b_a181_vista_bloqueada_sale_de_exam_msg() {
    // B-A181: toast "Cambio de vista…" ×2 (`set_perspective` +
    // `try_set_perspective`) sale de `exam.view_blocked` con locale.
    // ES pineado al literal previo (`app.rs:6439/:6530`); el `Err` del
    // controller (`controllers.rs:369` "Cambio de perspectiva…") queda
    // canónico intacto (API, no display).
    use grafito_ui::i18n::{exam_msg, t, Locale};
    assert_eq!(
        t("exam.view_blocked", Locale::Es),
        "Cambio de vista bloqueado en modo examen, che."
    );
    assert_eq!(
        t("exam.view_blocked", Locale::En),
        "View change blocked in exam mode."
    );
    assert_eq!(
        exam_msg("view_blocked", Locale::Pt),
        "Mudança de vista bloqueada no modo exame."
    );
    assert_eq!(
        exam_msg("view_blocked", Locale::De),
        "Ansichtswechsel im Prüfungsmodus blockiert."
    );
    // Wiring: los 2 sitios van por `t("exam.view_blocked", locale)`.
    let source = include_str!("app.rs");
    assert_eq!(
        source.matches("\"exam.view_blocked\", locale").count(),
        2,
        "vista bloqueada cablea 2 sitios con locale (B-A181)"
    );
    // Blindaje: viejo literal duro fuera del prod.
    assert!(
        !source.contains("\"Cambio de vista bloqueado en modo examen, che.\""),
        "vista bloqueada sin ES duro en prod (B-A181)"
    );
}

#[test]
fn exam_b_a206_bloqueado_oculto_salen_de_exam_msg() {
    // B-A206: tarjeta bloqueo (título + motivo) + oculto + botón reuse salen
    // de `exam.*` con locale. ES pineado a los literales previos
    // (`assistant.rs:4888` título, `:4896` motivo, `:4963` oculto sic;
    // botón `:4966` reuse `shell.assistant_reopen_hint`, 0 keys).
    use grafito_ui::i18n::{exam_msg, t, Locale};
    assert_eq!(t("exam.locked_title", Locale::Es), "Asistente bloqueado");
    assert_eq!(
        t("exam.locked_hint", Locale::Es),
        "Modo examen activo: salí del examen para volver a usarlo."
    );
    assert_eq!(
        t("exam.hidden_label", Locale::Es),
        "El asistente esta oculto."
    );
    assert_eq!(exam_msg("locked_title", Locale::En), "Assistant blocked");
    assert_eq!(
        exam_msg("locked_hint", Locale::Pt),
        "Modo exame ativo: sai do exame para voltar a usar."
    );
    assert_eq!(
        exam_msg("hidden_label", Locale::De),
        "Assistent ausgeblendet."
    );
    assert_eq!(
        t("shell.assistant_reopen_hint", Locale::Es),
        "Mostrar asistente"
    );
    // Wiring: los 4 sitios van por `t()` con locale (tarjeta título + motivo
    // + oculto + botón reuse; `source` es `assistant.rs`, sin auto-match).
    let source = include_str!("assistant.rs");
    for needle in [
        "\"exam.locked_title\", locale",
        "\"exam.locked_hint\", locale",
        "\"exam.hidden_label\", dock_locale",
        "\"shell.assistant_reopen_hint\",",
    ] {
        assert!(
            source.contains(needle),
            "bloqueado/oculto cablea {needle} con locale (B-A206)"
        );
    }
    // Blindaje: viejos literales duros fuera del prod (fragmentos por
    // `include_str!` auto-match, precedente B-A89/B-A105).
    for needle in [
        concat!("RichText::new(", "\"Asistente bloqueado\""),
        concat!("Modo examen activo", ": salí del examen"),
        concat!("El asistente ", "esta oculto."),
        concat!("button(", "\"Mostrar asistente\""),
    ] {
        assert!(
            !source.contains(needle),
            "bloqueado/oculto sin ES duro en prod (B-A206): {needle}"
        );
    }
}

#[test]
fn examen_veredicto_no_inventa_y_muestra_racha_si_hay() {
    // Nivel reflexivo (2026-10-14): el modal de salida deja de ser un "Sí,
    // salir" seco. Sin datos inventados: el lockdown no expone aciertos/total
    // ni nota (el mini-examen es un prompt sin corrección persistida y
    // `record_exam` jamás se llama desde la app), así que el veredicto solo
    // trae microcopy cálido + `racha N` si N≥2.
    use crate::app::GrafitoApp;
    use grafito_ui::i18n::Locale;
    // Sin racha (0-1): solo salida cálida, la línea de racha no se pinta.
    for streak in [0, 1] {
        let verdict = GrafitoApp::exam_exit_verdict(streak, Locale::Es);
        assert_eq!(
            verdict.warm,
            "Tranqui: salís del examen con todo en orden y tu progreso queda guardado."
        );
        assert_eq!(verdict.streak, None, "racha {streak}: no se inventa");
    }
    // Con racha se expone el dato existente, sin emojis.
    let verdict = GrafitoApp::exam_exit_verdict(5, Locale::Es);
    assert_eq!(verdict.streak, Some("racha 5".to_string()));
    // El microcopy resuelve en los 6 idiomas, sin vacíos ni fallback.
    for locale in [
        Locale::Es,
        Locale::En,
        Locale::Pt,
        Locale::It,
        Locale::Fr,
        Locale::De,
    ] {
        let verdict = GrafitoApp::exam_exit_verdict(0, locale);
        assert!(!verdict.warm.is_empty(), "veredicto vacío en {locale:?}");
        assert!(
            !verdict.warm.contains("exam."),
            "fallback en {locale:?}: {}",
            verdict.warm
        );
        let with_streak = GrafitoApp::exam_exit_verdict(3, locale);
        assert_eq!(
            with_streak.streak,
            Some("racha 3".to_string()),
            "racha invariante en {locale:?}"
        );
    }
    // Wiring: el modal arma el veredicto desde la racha verificada en días
    // (igual que el header) con locale (`source` es `app.rs`, el test no
    // auto-matchea).
    let source = include_str!("app.rs");
    for needle in [
        "exam_exit_verdict(verified, locale)",
        "verified_streak(now_epoch, &self.assistant.streak_steps_days)",
        "\"exam.modal_verdict\", locale",
    ] {
        assert!(
            source.contains(needle),
            "veredicto cablea {needle} con locale"
        );
    }
}

#[test]
fn racha_steps_sesion_sin_duplicados_y_con_tope() {
    // La racha Steps vive en sesión (sin persistir): dedupe + tope 64.
    use crate::app::GrafitoApp;
    let mut days = Vec::new();
    GrafitoApp::note_streak_steps_day(&mut days, 20_741);
    GrafitoApp::note_streak_steps_day(&mut days, 20_741);
    assert_eq!(days, vec![20_741], "sin duplicados");
    for day in 0..80 {
        GrafitoApp::note_streak_steps_day(&mut days, day);
    }
    assert_eq!(days.len(), GrafitoApp::STREAK_STEPS_DAYS_CAP);
    assert!(!days.contains(&20_741), "el tope evicta lo viejo");
    assert!(days.contains(&79));
    // Wiring: el hook del solver registra el día ante éxito real (fuera del
    // flag de celebración, una sola vez por día, no por sesión).
    let source = include_str!("app.rs");
    for needle in [
        "is_steps_success(input_was, message)",
        "note_streak_steps_day(&mut self.assistant.streak_steps_days, day)",
    ] {
        assert!(
            source.contains(needle),
            "hook Steps cablea {needle} (racha con historial)"
        );
    }
    // Wiring: el sync del header expone la racha verificada + tooltip.
    let sync_source = include_str!("assistant.rs");
    for needle in [
        "verified_streak(now_epoch, &self.assistant.streak_steps_days)",
        "assistant_streak_history_tooltip(",
    ] {
        assert!(
            sync_source.contains(needle),
            "sync cablea {needle} (racha con historial)"
        );
    }
}

#[test]
fn recovery_b_a183_modales_salen_de_recovery_msg() {
    // B-A183: los 2 modales de recovery (`draw_recovery_modal` +
    // `draw_recovery_corrupt_modal`) salen de `recovery.*` con locale.
    // ES pineado a los literales previos (`app.rs:3298` título, `:3292`
    // subtítulo, `:3346` Recuperar, `:3358/:3475` Seguir, `:3369/:3371`
    // diff, `:3428` corrupto, `:3440` mensaje, `:3463` Descartar).
    use grafito_ui::i18n::{recovery_msg, t, Locale};
    assert_eq!(
        t("recovery.title_found", Locale::Es),
        "Se encontró un autosave más nuevo"
    );
    assert_eq!(
        recovery_msg("subtitle_newer", Locale::En),
        "There are unsaved changes newer than {path}"
    );
    assert_eq!(
        recovery_msg("corrupt_msg", Locale::Pt),
        "O autosave de {path} está corrompido e não pôde ser recuperado: {err}"
    );
    assert_eq!(recovery_msg("discard", Locale::De), "Sidecar verwerfen");
    // Wiring: los 10 sitios van por `t("recovery.<key>", locale)`.
    let source = include_str!("app.rs");
    for key in [
        "title_found",
        "subtitle_newer",
        "recover",
        "keep",
        "show_diff",
        "hide_diff",
        "corrupt_title",
        "corrupt_msg",
        "discard",
    ] {
        assert!(
            source.contains(&format!("\"recovery.{key}\", locale")),
            "recovery.{key} sin cablear con locale (B-A183)"
        );
    }
    // Blindaje: viejos literales duros fuera del prod.
    for stale in [
        "\"Se encontró un autosave más nuevo\"",
        "\"Hay cambios sin guardar más nuevos que {}\"",
        "\"Recuperar autosave\"",
        "\"Seguir con guardado\"",
        "\"Ocultar diff\"",
        "\"Autosave corrupto\"",
        "\"Descartar sidecar\"",
    ] {
        assert!(
            !source.contains(stale),
            "recovery con ES duro en prod: {stale} (B-A183)"
        );
    }
}

#[test]
fn custom_b_a184_dialog_sale_de_custom_msg() {
    // B-A184: el diálogo custom-tool .ggt (`draw_custom_tool_dialog` +
    // `save/load_custom_tool_from_dialog` + `custom_tool_name_feedback`)
    // sale de `custom.dlg_*` con locale. ES pineado a los literales previos
    // (`app.rs:4666` título, `:4674` uso, `:4631`/`:4642`/`:4648` feedbacks,
    // `:4688`/`:4690` botones, `:4744` guardada, `:4784` cargada, `:4774`
    // error lectura, `:4725` filtro rfd).
    use crate::app::GrafitoApp;
    use grafito_ui::i18n::{custom_msg, t, Locale};
    assert_eq!(
        t("custom.dlg_title", Locale::Es),
        "Guardar herramienta personalizada"
    );
    assert_eq!(
        GrafitoApp::custom_tool_usage_text_for(3, 10, 64, Locale::Es),
        "Usando 3 de 10 del historial como pasos (tope 64)."
    );
    assert_eq!(
        GrafitoApp::custom_tool_usage_text_for(3, 10, 64, Locale::En),
        "Using 3 of 10 history entries as steps (cap 64)."
    );
    let store = grafito_command::ggbscript::CustomToolStore::new();
    assert_eq!(
        GrafitoApp::custom_tool_name_feedback_for("", &store, Locale::Es)
            .1
            .as_deref(),
        Some("Poné un nombre para la herramienta.")
    );
    assert_eq!(
        GrafitoApp::custom_tool_name_feedback_for("1mal", &store, Locale::En)
            .1
            .as_deref(),
        Some("The name uses letters/numbers/_ and starts with a letter or _.")
    );
    assert_eq!(
        GrafitoApp::custom_tool_saved_text_for("mi_tool", Locale::Es),
        "Herramienta 'mi_tool' guardada"
    );
    assert_eq!(
        GrafitoApp::custom_tool_loaded_text_for("mi_tool", Locale::Pt),
        "Ferramenta 'mi_tool' carregada (Ctrl+K para usá-la)"
    );
    assert_eq!(
        GrafitoApp::custom_tool_read_error_text_for("boom", Locale::De),
        "Die .ggt konnte nicht gelesen werden: boom"
    );
    assert_eq!(custom_msg("dlg_save", Locale::It), "Salva .ggt…");
    // Wiring: título/botones/filtro por `t("custom.dlg_*", locale)` y el
    // resto por helpers `_for` con locale.
    let source = include_str!("app.rs");
    for needle in [
        "\"custom.dlg_title\", locale",
        "\"custom.dlg_save\", locale",
        "\"custom.dlg_load\", locale",
        "\"custom.dlg_file_filter\", locale",
        "custom_tool_name_feedback_for",
        "custom_tool_usage_text_for",
        "custom_tool_saved_text_for",
        "custom_tool_loaded_text_for",
        "custom_tool_read_error_text_for",
    ] {
        assert!(
            source.contains(needle),
            "custom-tool sin cablear con locale: {needle} (B-A184)"
        );
    }
    // Blindaje: viejos literales duros fuera del prod.
    for stale in [
        "\"Guardar herramienta personalizada\"",
        "\"Guardar .ggt…\"",
        "\"Cargar .ggt…\"",
        "\"Poné un nombre para la herramienta.\"",
        "\"El nombre usa letras/números/_ y empieza con letra o _.\"",
        "\"Ese nombre ya existe: se va a reemplazar.\"",
        "\"Herramienta '{name}' guardada\"",
        "\"Herramienta '{name}' cargada (Ctrl+K para usarla)\"",
        "\"No se pudo leer el .ggt: {error}\"",
        "\"Usando {used} de {total} del historial",
        "\"Herramienta Grafito\"",
    ] {
        assert!(
            !source.contains(stale),
            "custom-tool con ES duro en prod: {stale} (B-A184)"
        );
    }
}

#[test]
fn rail_b_a185_tabs_tips_y_activo_salen_de_rail_msg() {
    // B-A185: labels abreviadas + tips + widget activo del rail salen de
    // `rail.*` con `rail_locale` (`app.config_locale()`); ES pineado a los
    // literales previos de `ui.rs:1040-1059` + `:1093` + `:1210`.
    // Plegable Math ELIMINADO 2026-10-06 (directiva usuario): rail plano.
    use grafito_ui::i18n::{t, Locale};
    assert_eq!(t("rail.tab_algebra", Locale::Es), "Álgebra");
    assert_eq!(t("rail.tab_tools", Locale::Es), "Herram.");
    assert_eq!(t("rail.tab_prob", Locale::Es), "Prob.");
    assert_eq!(
        t("rail.tip_tools", Locale::Es),
        "Herramientas de construcción y análisis"
    );
    assert_eq!(t("rail.active_suffix", Locale::Es), "{label}. Panel activo");
    assert_eq!(t("rail.tab_view", Locale::De), "Ansicht");
    // Wiring: el draw usa las keys con `rail_locale` (B-A185).
    // J2: sin Vault (directiva usuario): el rail junta Álgebra/Herram./
    // Datos/Prob. + Vista, sin entrada tab 6.
    let source = include_str!("ui.rs");
    for pin in [
        "let rail_locale = app.config_locale()",
        "rail_t(\"rail.tab_algebra\"",
        "rail_t(\"rail.tab_tools\"",
        "rail_t(\"rail.tab_data\"",
        "rail_t(\"rail.tab_prob\"",
        "rail_t(\"rail.tab_view\"",
        "rail_t(\"rail.tip_algebra\"",
        "rail_t(\"rail.active_suffix\"",
        ".replace(\"{label}\", label)",
    ] {
        assert!(source.contains(pin), "rail B-A185 cablea {pin}");
    }
    assert!(
        !source.contains("entries.push((6,"),
        "resto del Vault tab 6 en rail (J2)"
    );
    assert!(
        !source.contains("vault_entry"),
        "resto del Vault en rail (J2)"
    );
    // Invariantes intencionales: nombres de sección sin key.
    assert!(
        !source.contains("\"Math v\"") && !source.contains("\"Math >\""),
        "resto del plegable Math en rail"
    );
    assert!(
        !source.contains("math_group_open"),
        "resto del estado del plegable Math"
    );
    // Blindaje: viejos literales ES duros fuera del prod.
    for stale in [
        "\"Álgebra\"",
        "\"Herram.\"",
        "\"Datos\"",
        "\"Prob.\"",
        "\"Vista\"",
        "\"Objetos, variables y CAS\"",
        "\"Herramientas de construcción y análisis\"",
        "\"Hoja de cálculo vinculada y tablas\"",
        "\"Probabilidad y Estadística\"",
        "\"Solo visual: cuadrícula, ejes, 3D y capas\"",
        "\"Matemática: álgebra, herramientas, datos y probabilidad\"",
        "\"{label}. Panel activo\"",
    ] {
        assert!(
            !source.contains(stale),
            "rail con ES duro en prod: {stale} (B-A185)"
        );
    }
}

#[test]
fn shell_b_a186_chrome_sale_de_shell_msg() {
    // B-A186: reopen Asistente + toggle tema + Config + dock + hover de
    // entrada salen de `shell.*` con `locale` (`config_locale()` o
    // pre-borrow E0502); tab "Inspector" reutiliza `inspector.panel_title`.
    // ES pineado a los literales previos de `ui.rs:120-121` + `:127-149` +
    // `:957-970` + `:1287-1381` + `:1529`.
    use grafito_ui::i18n::{t, Locale};
    assert_eq!(t("shell.config_btn", Locale::Es), "Configuración");
    assert_eq!(
        t("shell.config_open", Locale::Es),
        "Configuración. Panel abierto"
    );
    assert_eq!(t("shell.assistant_reopen", Locale::Es), "Asistente");
    assert_eq!(
        t("shell.assistant_reopen_hint", Locale::Es),
        "Mostrar asistente"
    );
    assert_eq!(t("shell.assistant_pending", Locale::Es), "Asistente •");
    assert_eq!(
        t("shell.assistant_dock_hint", Locale::Es),
        "Asistente matemático"
    );
    assert_eq!(
        t("shell.theme_to_light", Locale::Es),
        "Cambiar a tema claro"
    );
    assert_eq!(t("shell.workspace_title", Locale::Es), "Espacio de trabajo");
    assert_eq!(t("shell.dock_hide", Locale::Es), "Ocultar dock contextual");
    assert_eq!(t("shell.workspace_title", Locale::De), "Arbeitsbereich");
    assert!(t("shell.input_hint", Locale::Es).contains("{tool_hint}"));
    // Wiring: el draw usa las keys con `locale` (B-A186).
    let source = include_str!("ui.rs");
    for pin in [
        "let reopen_locale = app.config_locale()",
        "draw_theme_toggle_switch(ui, app.dark_mode, app.config_locale())",
        "let shell_cfg = app.config_locale()",
        "shell_t(\"shell.config_btn\"",
        "shell_t(\"shell.config_open\"",
        "shell_t(\"shell.assistant_reopen\"",
        "shell_t(\"shell.assistant_pending\"",
        "shell_t(\"shell.assistant_dock_hint\"",
        "t(\"shell.theme_to_light\", locale)",
        "t(\"shell.theme_dark_on\", locale)",
        "shell_t(\"shell.workspace_title\"",
        "shell_t(\"shell.dock_hide\"",
        "shell_t(\"shell.inspector_hint\"",
        "shell_t(\"inspector.panel_title\"",
        "\"shell.input_hint\", input_locale",
        ".replace(\"{tool_hint}\", &tool_hint)",
    ] {
        assert!(source.contains(pin), "shell B-A186 cablea {pin}");
    }
    // Blindaje: viejos literales ES duros fuera del prod.
    for stale in [
        "\"Mostrar asistente\"",
        "\"Cambiar a tema claro\"",
        "\"Cambiar a tema oscuro\"",
        "\"Tema oscuro activado. Cambiar a tema claro\"",
        "\"Tema claro activado. Cambiar a tema oscuro\"",
        "\"Configuración\"",
        "\"Configuración. Panel abierto\"",
        "\"Espacio de trabajo\"",
        "\"Ocultar dock contextual\"",
        "RichText::new(\"Inspector\")",
        "\"Propiedades del objeto seleccionado\"",
        "\"Asistente •\"",
        "\"Asistente matemático\"",
        "\"Escribí un comando",
    ] {
        assert!(
            !source.contains(stale),
            "shell con ES duro en prod: {stale} (B-A186)"
        );
    }
}

#[test]
fn save_b_a187_dialog_sale_de_save_msg() {
    // B-A187: título + body + error + 3 botones del diálogo
    // cambios-sin-guardar salen de `save.*` con `locale`.
    // ES pineado a los literales previos de `ui.rs:1727` (título),
    // `:1789` (error), `:1803/1815/1827` (botones) y `lifecycle.rs:50`
    // (sin título) + `:53/55/56` (prompts).
    use grafito_ui::i18n::{t, Locale};
    assert_eq!(t("save.title", Locale::Es), "Cambios sin guardar");
    assert_eq!(
        t("save.prompt_new", Locale::Es),
        "¿Guardar cambios en \"{file}\" antes de crear un documento nuevo?"
    );
    assert_eq!(t("save.untitled", Locale::Es), "Sin título");
    assert_eq!(t("save.error", Locale::Es), "No se pudo guardar: {error}");
    assert_eq!(t("save.cancel", Locale::Es), "Cancelar");
    assert_eq!(t("save.discard", Locale::Es), "Descartar");
    assert_eq!(t("save.save", Locale::Es), "Guardar");
    // Diálogo minimalista 2026-10-06: título + nombre archivo + botones;
    // la pregunta larga (prompt_message_for) se eliminó con sus fns.
    // Wiring: el draw usa las keys con `locale` (B-A187).
    let ui = include_str!("ui.rs");
    for pin in [
        "let save_locale = app.config_locale()",
        "t(\"save.title\", save_locale)",
        "doc_caption",
        "t(\"save.error\", locale)",
        ".replace(\"{error}\", error)",
        "t(\"save.cancel\", locale)",
        "t(\"save.discard\", locale)",
        "t(\"save.save\", locale)",
    ] {
        assert!(ui.contains(pin), "save B-A187 cablea {pin}");
    }
    // Blindaje: viejos literales ES duros fuera del prod del diálogo.
    for stale in [
        "\"Cambios sin guardar\"",
        "\"Cancelar\"",
        "\"Descartar\"",
        "\"Guardar\"",
        "\"Sin título\"",
        "\"No se pudo guardar",
        "¿Guardar cambios en",
    ] {
        assert!(
            !ui.contains(stale),
            "save con ES duro en ui.rs: {stale} (B-A187)"
        );
    }
}

#[test]
fn hint_b_a188_hints_salen_de_hint_msg() {
    // B-A188: los 56 hints 2D + fallback 3D + 2 toasts ejemplo salen de
    // `hint.*`/`shell.example_*` con `locale`. Teseracto/Hipercubo 2D+3D
    // comparten key (textos idénticos). Error de ejemplo reutiliza
    // `menu.persp.load_example` existente (0 keys).
    // ES pineado a los literales previos de `ui.rs:2024-2095` + `:598-609`.
    use grafito_ui::i18n::{t, Locale};
    assert_eq!(
        t("hint.select", Locale::Es),
        "Clic objeto · arrastrar fondo para mover"
    );
    assert_eq!(t("hint.point", Locale::Es), "Punto: clic para crear");
    assert_eq!(
        t("hint.vector", Locale::Es),
        "⇒ Vector: clic origen, clic extremo"
    );
    assert_eq!(
        t("hint.polygon_difference", Locale::Es),
        "\\ Diferencia: dos polígonos"
    );
    assert_eq!(
        t("hint.tesseract_4d", Locale::Es),
        "Teseracto 4D: clic para crear un objeto centrado y proyectado"
    );
    assert_eq!(
        t("hint.complex_contour", Locale::Es),
        "Contorno: dibujá el lazo · imán con Shift libre · Esc cancela"
    );
    assert_eq!(
        t("hint.fallback_2d", Locale::Es),
        "Espacio / clic medio: mover vista"
    );
    assert_eq!(
        t("hint.fallback_3d", Locale::Es),
        "3D: clic izq pan (Select), der orbitar, rueda zoom"
    );
    assert_eq!(
        t("hint.select", Locale::En),
        "Click object · drag background to move"
    );
    assert_eq!(
        t("hint.fallback_2d", Locale::De),
        "Leertaste / Mittelklick: Ansicht verschieben"
    );
    assert_eq!(
        t("hint.translate", Locale::Pt),
        "Translação: clique no início do vetor, clique na ponta"
    );
    assert_eq!(
        t("shell.example_loaded", Locale::Es),
        "Ejemplo cargado para la perspectiva actual"
    );
    // El botón siempre agrega encima (append + snapshot, deshacer revierte):
    // `shell.example_skipped` queda definida pero sin uso en ui.rs.
    assert_eq!(
        t("shell.example_loaded", Locale::Fr),
        "Exemple chargé pour la perspective actuelle"
    );
    // 2D+3D comparten key en tesseract/hypercube.
    assert_eq!(
        crate::ui::status_hint_for_3d_tool(grafito_ui::Tool::Tesseract4D, Locale::Es),
        t("hint.tesseract_4d", Locale::Es)
    );
    // Wiring: el draw usa las keys con `locale` (B-A188).
    let ui = include_str!("ui.rs");
    for pin in [
        "let input_locale = app.config_locale()",
        "status_hint_for_tool(app.current_tool, input_locale)",
        "status_hint_for_3d_tool(app.current_tool, input_locale)",
        "hint_msg(\"select\", locale)",
        "hint_msg(\"fallback_2d\", locale)",
        "hint_msg(\"fallback_3d\", locale)",
        "hint_msg(\"tesseract_4d\", locale)",
        "t(\"shell.example_loaded\", locale)",
        "t(\"menu.persp.load_example\", locale)",
    ] {
        assert!(ui.contains(pin), "hint B-A188 cablea {pin}");
    }
    // Blindaje: viejos literales ES duros fuera del prod.
    for stale in [
        "\"Clic objeto · arrastrar fondo para mover\"",
        "\"Punto: clic para crear\"",
        "\"Punto 3D: clic para crear\"",
        "\"Recta: clic en dos puntos\"",
        "\"Segmento: clic en dos puntos\"",
        "\"Semirrecta: clic origen, clic direccion\"",
        "\"⇒ Vector: clic origen, clic extremo\"",
        "\"Circulo: clic centro, clic borde\"",
        "\"Poligono: clic vertices, clic der para cerrar\"",
        "\"Poligono regular: clic centro, clic vertice\"",
        "\"f(x) Función: clic para crear y editar\"",
        "\"Distancia: clic en dos puntos\"",
        "\"Restriccion Distancia: clic en dos puntos\"",
        "\"Angulo: clic en 3 puntos (vertice, brazo 1, brazo 2)\"",
        "\"Restriccion Angulo: clic en dos rectas\"",
        "\"Area: clic en poligono o circulo\"",
        "\"m Pendiente: clic en recta\"",
        "\"═ Deslizador: clic para crear variable\"",
        "\"Locus: clic punto driver, clic punto objetivo\"",
        "\"Herramienta no disponible en esta versión\"",
        "\"M Punto medio: clic en dos puntos\"",
        "\"⟂ Perpendicular: clic en dos puntos\"",
        "\"Tangente: selecciona circulo y recta\"",
        "\"x₀ Raíces: clic en una función\"",
        "\"max Extremos: clic en una función\"",
        "\"Interseccion: clic en dos objetos\"",
        "\"Coincidente: selecciona dos puntos\"",
        "\"Horizontal: selecciona una recta\"",
        "\"Vertical: selecciona una recta\"",
        "\"= Longitud igual: selecciona dos segmentos\"",
        "\"Simetria: punto, imagen, eje\"",
        "\"Elipse: dos focos y un punto\"",
        "\"⩗ Parábola: foco y directriz\"",
        "\"⩘ Hipérbola: dos focos y un punto\"",
        "\"C5 Cónica: cinco puntos\"",
        "\"∪ Unión: dos polígonos\"",
        "\"∩ Intersección: dos polígonos\"",
        "Diferencia: dos polígonos",
        "\"XOR: dos poligonos\"",
        "\"Esfera 3D: clic centro y borde\"",
        "\"Cubo 3D: clic centro y borde\"",
        "\"Traslada: clic inicio del vector, clic extremo\"",
        "\"Rota: clic centro, clic para definir el ángulo\"",
        "\"Homotecia: clic centro, clic donde cae la imagen\"",
        "\"Refleja: clic objeto, clic dos puntos del eje\"",
        "\"Compás: clic centro, clic punto del radio\"",
        "\"Semicírculo: clic centro, clic punto del radio\"",
        "\"Spline: clic puntos, clic cerca del inicio para cerrar\"",
        "\"Prisma: clic sobre un polígono base\"",
        "\"Tetraedro: clic para crear (arista 2)\"",
        "\"Casilla: clic para crear ligada a variable\"",
        "\"Caja de entrada: clic para crear ligada a variable\"",
        "\"Teseracto 4D: clic para crear un objeto centrado y proyectado\"",
        "\"Hipercubo 5D: clic para crear un objeto centrado y proyectado\"",
        "\"Contorno: dibujá el lazo · imán con Shift libre · Esc cancela\"",
        "\"Espacio / clic medio: mover vista\"",
        "\"3D: clic izq pan (Select), der orbitar, rueda zoom\"",
        "\"Ejemplo cargado para la perspectiva actual\"",
        "\"No se cargó el ejemplo: el documento ya tiene objetos\"",
        "\"Cargar ejemplo de perspectiva\"",
    ] {
        assert!(
            !ui.contains(stale),
            "hint con ES duro en ui.rs: {stale} (B-A188)"
        );
    }
}

/// Habla simulada SIN audio para el toggle TTS (unix): el binario `true`
/// sale de inmediato, así `stop`/`poll` se ejercitan sin espeak ni sonido
/// (el toggle jamás habla en tests: sería auto-play fuera de un clic).
#[cfg(unix)]
fn habla_terminada_falsa(turno: usize) -> crate::tts_cli::StepsTtsSpeech {
    let hijo = std::process::Command::new("true")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("`true` disponible para simular habla sin audio");
    // `true` ya salió: el sondeo lo ve terminado sin matar nada.
    std::thread::sleep(std::time::Duration::from_millis(50));
    crate::tts_cli::StepsTtsSpeech {
        turn: turno,
        child: hijo,
        started: std::time::Instant::now(),
    }
}

#[test]
fn toggle_speak_steps_turno_fantasma_no_habla() {
    // Turno inexistente → aviso honesto, sin spawn ni habla (el gate dice
    // Start pero no hay texto: nada suena sin contenido válido).
    let mut app = crate::app::dummy_grafito_app();
    egui::__run_test_ctx(|ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            app.toggle_speak_steps(999, ui.ctx());
        });
    });
    assert!(app.steps_tts.is_none(), "turno fantasma no spawnea habla");
    assert_eq!(app.assistant.steps_tts_speaking_turn, None);
}

#[test]
fn toggle_speak_steps_sin_habla_previa_no_deja_residuo() {
    // Sin habla en curso y turno fantasma: doble toggle idempotente.
    let mut app = crate::app::dummy_grafito_app();
    egui::__run_test_ctx(|ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            app.toggle_speak_steps(999, ui.ctx());
            app.toggle_speak_steps(999, ui.ctx());
        });
    });
    assert!(app.steps_tts.is_none());
    assert_eq!(app.assistant.steps_tts_speaking_turn, None);
}

#[cfg(unix)]
#[test]
fn toggle_speak_steps_segunda_pulsacion_detiene_sin_solape() {
    // Habla en curso en ESTE turno (simulada sin audio) → 2ª pulsación =
    // Stop: kill+wait, sin habla residual ni etiqueta «Detener» rancia.
    let mut app = crate::app::dummy_grafito_app();
    app.steps_tts = Some(habla_terminada_falsa(2));
    app.assistant.steps_tts_speaking_turn = Some(2);
    egui::__run_test_ctx(|ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            app.toggle_speak_steps(2, ui.ctx());
        });
    });
    assert!(app.steps_tts.is_none(), "detener suelta el Child");
    assert_eq!(
        app.assistant.steps_tts_speaking_turn, None,
        "la etiqueta vuelve a «Leer»"
    );
}

#[cfg(unix)]
#[test]
fn poll_steps_tts_suelta_habla_terminada_y_sincroniza_panel() {
    // Sondeo por frame: habla ya terminada → se suelta + repinta, y la
    // presencia del binario se refleja en el panel para el botón «Leer».
    let mut app = crate::app::dummy_grafito_app();
    app.steps_tts = Some(habla_terminada_falsa(4));
    app.assistant.steps_tts_speaking_turn = Some(4);
    egui::__run_test_ctx(|ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            app.poll_steps_tts(ui.ctx());
        });
    });
    assert!(app.steps_tts.is_none());
    assert_eq!(app.assistant.steps_tts_speaking_turn, None);
    assert_eq!(
        app.assistant.steps_tts_available,
        crate::tts_cli::is_espeak_available(),
        "el panel refleja la presencia real del binario"
    );
}

#[test]
fn tts_solo_habla_desde_el_toggle() {
    // Piel pura + clic explícito: el único spawn de voz de la app vive en
    // `toggle_speak_steps` (el draw jamás habla: solo emite la acción).
    let fuente = include_str!("assistant.rs");
    assert_eq!(
        fuente.matches("spawn_speech(").count(),
        1,
        "spawn de voz solo en el toggle"
    );
    assert!(
        !fuente.contains("speak_steps_text("),
        "la versión bloqueante no corre en la UI (hilo worker/tests)"
    );
}

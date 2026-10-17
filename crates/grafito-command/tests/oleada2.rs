#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Oleada 2 P4: M factible con entrada UI visible (paleta + e2e).
//! Regla de oro: cada comando nuevo visible en paleta + responde por
//! `process_input` + test e2e. Nada muerto.

use grafito_command::{
    command_registry,
    commands::{process_input, CommandOutcome},
};
use grafito_core::{Document, GeoObject, LineObj};

fn run(document: &mut Document, command: &str) -> CommandOutcome {
    let mut input = command.to_owned();
    process_input(document, &mut input)
}

fn assert_message_contains(document: &mut Document, command: &str, needle: &str) {
    match run(document, command) {
        CommandOutcome::Message(message) => assert!(
            message.contains(needle),
            "{command} → {message} (esperaba '{needle}')"
        ),
        CommandOutcome::Ok => panic!("{command} dio Ok, esperaba Message con '{needle}'"),
        CommandOutcome::Error(message) => {
            panic!("{command} dio Error: {message} (esperaba '{needle}')")
        }
    }
}

fn assert_ok_or_message(document: &mut Document, command: &str) {
    match run(document, command) {
        CommandOutcome::Ok | CommandOutcome::Message(_) => {}
        CommandOutcome::Error(message) => panic!("{command} dio Error: {message}"),
    }
}

fn palette_must_be_visible(canonical: &str) {
    let spec = command_registry::resolve(canonical)
        .unwrap_or_else(|| panic!("{canonical} debe estar registrado"));
    assert!(
        spec.palette_visible,
        "{canonical} debe ser visible en paleta"
    );
    assert_eq!(spec.canonical, canonical);
}

#[allow(dead_code)]
fn first_label_matching(document: &Document, pred: impl Fn(&GeoObject) -> bool) -> String {
    document
        .objects_iter()
        .find_map(|(_, obj)| {
            if pred(obj) {
                Some(obj.label().to_string())
            } else {
                None
            }
        })
        .expect("fixture no creó el objeto esperado")
}

fn parse_xy(xy: &str) -> (f64, f64) {
    let t = xy.trim().trim_start_matches('(').trim_end_matches(')');
    let mut it = t.split(',');
    let x: f64 = it.next().unwrap_or("0").trim().parse().unwrap_or(f64::NAN);
    let y: f64 = it.next().unwrap_or("0").trim().parse().unwrap_or(f64::NAN);
    (x, y)
}

fn make_point(document: &mut Document, xy: &str) -> String {
    assert_ok_or_message(document, &format!("Point[{xy}]"));
    let (ex, ey) = parse_xy(xy);
    document
        .objects_iter()
        .find_map(|(_, obj)| match obj {
            GeoObject::Point(p)
                if (p.position.x - ex).abs() < 1e-9 && (p.position.y - ey).abs() < 1e-9 =>
            {
                Some(p.label.clone())
            }
            _ => None,
        })
        .expect("punto fixture por coordenadas")
}

fn make_line(document: &mut Document, a: &str, b: &str) -> String {
    assert_ok_or_message(document, &format!("Line[{a},{b}]"));
    let (ax, ay) = parse_xy(a);
    let (bx, by) = parse_xy(b);
    // La recta guarda start/end normalizados; busca por dirección y punto medio.
    document
        .objects_iter()
        .filter_map(|(_, obj)| match obj {
            GeoObject::Line(l) => Some((l.label.clone(), l.start, l.end)),
            _ => None,
        })
        .find(|(_, s, e)| {
            let mid = ((s.x + e.x) * 0.5, (s.y + e.y) * 0.5);
            let exp_mid = ((ax + bx) * 0.5, (ay + by) * 0.5);
            (mid.0 - exp_mid.0).abs() < 1e-6 && (mid.1 - exp_mid.1).abs() < 1e-6
        })
        .map(|(label, _, _)| label)
        .expect("recta fixture por punto medio")
}

#[test]
fn oleada2_todos_visibles_en_paleta() {
    for canonical in [
        "Eigenvalues",
        "Eigenvectors",
        "Uniform",
        "Exponential",
        "ChiSquared",
        "InverseExponential",
        "InverseUniform",
        "CFactor",
        "CIFactor",
        "PartialFractions",
        "AreCollinear",
        "AreConcurrent",
        "AreConcyclic",
        "AreParallel",
        "ArePerpendicular",
        "SetCaption",
        "SetLineStyle",
        "SetPointStyle",
        "SetLayer",
        "Dodecahedron",
        "Icosahedron",
        "Octahedron",
        "InfiniteCone",
        "InfiniteCylinder",
    ] {
        palette_must_be_visible(canonical);
    }
    // Frente trig+racionalización (+4 visibles S): 287 → 291, 333 → 337.
    // Frente R3.1 (Rename stub→visible): 291 → 292 (total 337 intacto).
    // Frente 3D-A2 (Vista3D visible S): 292 → 293, 337 → 338.
    // Frente P0.2 (álgebra CAS: +41 visibles S): 293 → 334, 338 → 379.
    // Frente P0.3 (complejo: +7 visibles S): 334 → 341, 379 → 386.
    // Frente P0.4 (demostración: +3 visibles S): 341 → 344, 386 → 389.
    // Frente P0.5 (optimización+gráficos: +8 visibles S): 344 → 352, 389 → 397.
    // Frente P1 (listas+stats: +82 visibles S): 352 → 434, 397 → 479.
    // Frente P1b (constructor List: +1 visible S): 434 → 435, 479 → 480.
    // Frente P2 (medidas/geometría: +25 visibles S): 435 → 460, 480 → 505.
    // Frente P3b (scripting: +11 visibles S + Execute real): 460 → 472, 505 → 516.
    // Frente P4 (CAS+listas/geometría/stats/display: +105 visibles S): 472 → 577, 516 → 621.
    // Frente P4-fix: Shear/Stretch/FractionText/SurdText visibles: 577 → 581.
    // Frente P5 (cierre nominal: +18 visibles S): 581 → 599, 621 → 639.
    // Ola 0.2 (Groebner/GroebnerLex: +2 visibles S): 599 → 601, 639 → 641.
    // Ola 0.3 (StartAnimation/StopAnimation/Delete visibles: +3): 601 → 604.
    // Ola 2.3 (ZTest2/FTest visibles): 604 → 606, 641 → 643.
    // Harness Fase A (+7 visibles S): 606 → 613, 643 → 650.
    // Frente fantasma+nuevos (+12 visibles S): 613 → 625, 650 → 662.
    // Frente fantasma-2 (+71 visibles S): 625 → 696, 662 → 733.
    // Frente G1 (contrato grafito-geometry: +14 visibles S — CAS +9, AM2 +3,
    // Texto +2): 696 → 710, 733 → 747.
    // O1-cierre (EDPs completas: Transport1D + HeatSeparation, AM2 +2):
    // 710 → 712 visibles, 747 → 749 totales.
    // O3-cierre (Wolfram bidireccional: ParseWolfram + ToWolfram, Texto +2):
    // 712 → 714 visibles, 749 → 751 totales.
    // O5-cierre (GaussKronrod + FFT + BfgsMinimize + BigInt + BigRational):
    // 714 → 719 visibles, 751 → 756 totales.
    // O6-cierre (8 especiales CAS): 719 → 727 visibles, 756 → 764 totales.
    // O4.1 (3 curvas): 727 → 730, 764 → 767. O4.2 (4 no-paramétricos):
    // 730 → 734, 767 → 771. O4.3a (3 complejos): 734 → 737, 771 → 774.
    // O4.4 (ANOVA2): 737 → 738, 774 → 775. O4.5 (NPV + IRR): 738 → 740,
    // 775 → 777. O4.3b (3 Möbius): 740 → 743, 777 → 780.
    // O7 (3 Riemann): 743 → 746, 780 → 783.
    // Física visual (+4 Dinámica: CampoCargas, TrayectoriaProyectil,
    // OsciladorArmonico, CargaCapacitor): 746 → 750, 783 → 787.
    // Equipotenciales (+1 Dinámica): 750 → 751, 787 → 788.
    // VectoresProyectil (+1 Dinámica): 751 → 752, 788 → 789.
    // CampoHilo (+1 Dinámica): 752 → 753, 789 → 790.
    // FemInducida (+1 Dinámica): 753 → 754, 790 → 791.
    // BarraHooke (+1 Dinámica): 754 → 755, 791 → 792.
    // ConduccionTermica (+1 Dinámica): 755 → 756, 792 → 793.
    // Gravitación (+2 Dinámica): 756 → 758, 793 → 795.
    // TuboVenturi (+1 Dinámica): 758 → 759, 795 → 796.
    // RayoRefraccion (+1 Dinámica): 759 → 760, 796 → 797.
    // OndaEstacionaria (+1 Dinámica): 760 → 761, 797 → 798.
    // GasIdeal (+1 Dinámica): 761 → 762, 798 → 799.
    // ResonanciaRLC (+1 Dinámica): 762 → 763, 799 → 800.
    // EfectoDoppler (+1 Dinámica): 763 → 764, 800 → 801.
    // DobleRendija (+1 Dinámica): 764 → 765, 801 → 802.
    // DiagramaMinkowski (+1 Dinámica): 765 → 766, 802 → 803.
    // Desintegracion (+1 Dinámica): 766 → 767, 803 → 804.
    // EfectoFotoelectrico (+1 Dinámica): 767 → 768, 804 → 805.
    // Pendulo (+1 Dinámica): 768 → 769, 805 → 806.
    // CicloCarnot (+1 Dinámica): 769 → 770, 806 → 807.
    // LenteDelgada (+1 Dinámica): 770 → 771, 807 → 808.
    // CaidaRozamiento (+1 Dinámica): 771 → 772, 808 → 809.
    // EfectoCompton (+1 Dinámica): 772 → 773, 809 → 810.
    // PotencialGravitatorio (+1 Dinámica): 773 → 774, 810 → 811.
    // EspejoEsferico (+1 Dinámica): 774 → 775, 811 → 812.
    // LeyMalus (+1 Dinámica): 775 → 776, 812 → 813.
    // LeyWien (+1 Dinámica): 776 → 777, 813 → 814.
    // PotenciaStefan (+1 Dinámica): 777 → 778, 814 → 815.
    // PresionHidrostatica (+1 Dinámica): 778 → 779, 815 → 816.
    // EmpujeArquimedes (+1 Dinámica): 779 → 780, 816 → 817.
    // NumeroReynolds (+1 Dinámica): 780 → 781, 817 → 818.
    // DilatacionLineal (+1 Dinámica): 781 → 782, 818 → 819.
    // PlanoInclinado (+1 Dinámica): 782 → 783, 819 → 820.
    // LeyOhm (+1 Dinámica): 783 → 784, 820 → 821.
    // CalorEspecifico (+1 Dinámica): 784 → 785, 821 → 822.
    // FuerzaCoulomb (+1 Dinámica): 785 → 786, 822 → 823.
    // LeyEnfriamiento (+1 Dinámica): 786 → 787, 823 → 824.
    // OndaDeBroglie (+1 Dinámica): 787 → 788, 824 → 825.
    // EnergiaMecanica (+1 Dinámica): 788 → 789, 825 → 826.
    // FuerzaLorentz (+1 Dinámica): 789 → 790, 826 → 827.
    // MovimientoAcelerado (+1 Dinámica): 790 → 791, 827 → 828.
    // OrbitaCircular (+1 Dinámica): 791 → 792, 828 → 829.
    // NivelSonoro (+1 Dinámica): 792 → 793, 829 → 830.
    // PH (+1 Dinámica): 793 → 794, 830 → 831.
    // Densidad (+1 Dinámica): 794 → 795, 831 → 832.
    // Trabajo (+1 Dinámica): 795 → 796, 832 → 833.
    // PotenciaMecanica (+1 Dinámica): 796 → 797, 833 → 834.
    // TerceraKepler (+1 Dinámica): 797 → 798, 834 → 835.
    // FuerzaRozamiento (+1 Dinámica): 798 → 799, 835 → 836.
    // CampoCargas3D (+1 Dinámica): 799 → 800, 836 → 837.
    // SistemaOrbital (+1 Dinámica lab 3D #2): 800 → 801, 837 → 838.
    // VelocidadOnda (+1 Dinámica onda viajera): 801 → 802, 838 → 839.
    // CantidadMovimiento (+1 Dinámica p=m·v): 802 → 803, 839 → 840.
    // AceleracionCentripeta (+1 Dinámica a=v²/r): 803 → 804, 840 → 841.
    // LeyGravitacion (+1 Dinámica F=Gm1m2/r²): 804 → 805, 841 → 842.
    // EnergiaBohr (+1 Dinámica E_n=−13.6/n²): 805 → 806, 842 → 843.
    // LeyFaraday (+1 Dinámica fem=−N·ΔΦ/Δt): 806 → 807, 843 → 844.
    // FlujoGauss (+1 Dinámica Φ=Q/ε₀): 807 → 808, 844 → 845.
    // CalorLatente (+1 Dinámica Q=m·L): 808 → 809, 845 → 846.
    // ChoqueElastico (+1 Dinámica choque 1D): 809 → 810, 846 → 847.
    // VelocidadSonido (+1 Dinámica v aire): 810 → 811, 847 → 848.
    // AlcanceProyectil (+1 Dinámica tiro): 811 → 812, 848 → 849.
    // EnergiaCapacitor (+1 Dinámica ½CU²): 812 → 813, 849 → 850.
    // paleta_fisica_armonica_semantica: ancla estética (docs/paleta-fisica.md).
    // SolveODE1/SolveODEHomog (+2 CAS con spec): 813 → 815, 850 → 852.
    assert_eq!(command_registry::palette_commands().count(), 815);
    assert_eq!(command_registry::all().len(), 852);
}

#[test]
fn oleada2_eigen_y_proba() {
    let mut document = Document::new();
    assert_message_contains(
        &mut document,
        "Eigenvalues[[[1, 0], [0, 2]]]",
        "Eigenvalues",
    );
    assert_message_contains(&mut document, "Eigenvectors[[[1, 0], [0, 2]]]", "lambda");
    // Valores conocidos: diagonal 1,2 → autovalores 1 y 2.
    match run(&mut document, "Eigenvalues[[[1, 0], [0, 2]]]") {
        CommandOutcome::Message(m) => {
            assert!(m.contains('1') && m.contains('2'), "autovalores 1,2: {m}");
        }
        other => panic!("Eigenvalues debe dar Message, dio {other:?}"),
    }
    assert_message_contains(&mut document, "Uniform[0, 1]", "PDF");
    assert_message_contains(&mut document, "Uniform[0, 1, 0.5]", "CDF");
    assert_message_contains(&mut document, "Exponential[1]", "PDF");
    assert_message_contains(&mut document, "Exponential[1, 0.5]", "CDF");
    assert_message_contains(&mut document, "ChiSquared[2]", "PDF");
    assert_message_contains(&mut document, "ChiSquared[2, 1]", "CDF");
    assert_message_contains(&mut document, "InverseExponential[0.5, 1]", "0.693");
    assert_message_contains(&mut document, "InverseUniform[0.5, 0, 10]", "5.000000");
    // Errores honestos.
    match run(&mut document, "Exponential[0]") {
        CommandOutcome::Error(m) => assert!(m.contains("Exponential"), "{m}"),
        other => panic!("Exponential[0] debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "InverseUniform[2, 0, 1]") {
        CommandOutcome::Error(m) => assert!(m.contains("InverseUniform"), "{m}"),
        other => panic!("p fuera de rango debe dar Error, dio {other:?}"),
    }
}

#[test]
fn oleada2_cfactor_y_parciales() {
    let mut document = Document::new();
    // x^2+1 → complejo conjugado con i.
    assert_message_contains(&mut document, "CFactor[x^2+1, x]", "i");
    // Real sigue igual que Factor.
    assert_message_contains(&mut document, "CFactor[x^2-1, x]", "x");
    // Gaussiana: contenido entero + complejo.
    assert_message_contains(&mut document, "CIFactor[2*x^2+2, x]", "i");
    // Parciales con verificación: (2x+3)/((x-1)(x+2)) → A/(x-1)+B/(x+2).
    assert_message_contains(
        &mut document,
        "PartialFractions[(2*x+3)/((x-1)*(x+2))]",
        "/",
    );
    match run(&mut document, "PartialFractions[(2*x+3)/((x-1)*(x+2))]") {
        CommandOutcome::Message(m) => {
            assert!(m.contains("x - 1") || m.contains("x-1"), "lineal 1: {m}");
            assert!(m.contains("x + 2") || m.contains("x+2"), "lineal 2: {m}");
        }
        other => panic!("PartialFractions debe dar Message, dio {other:?}"),
    }
    // Cuadrática irreducible: 1/(x^2+1) → (Bx+C)/(x^2+1).
    assert_message_contains(&mut document, "PartialFractions[1/(x^2+1)]", "x^2");
    // Impropia honesta (grado num 3 ≥ grado den 2).
    match run(&mut document, "PartialFractions[(x^3+1)/(x^2+1)]") {
        CommandOutcome::Error(m) => assert!(m.contains("propia"), "{m}"),
        other => panic!("impropia debe dar Error honesto, dio {other:?}"),
    }
}

#[test]
fn oleada2_relaciones() {
    let mut document = Document::new();
    let a = make_point(&mut document, "(0, 0)");
    let b = make_point(&mut document, "(1, 1)");
    let c = make_point(&mut document, "(2, 2)");
    let d = make_point(&mut document, "(0, 1)");
    assert_message_contains(
        &mut document,
        &format!("AreCollinear[{a}, {b}, {c}]"),
        "true",
    );
    assert_message_contains(
        &mut document,
        &format!("AreCollinear[{a}, {b}, {d}]"),
        "false",
    );
    // Paralelas horizontales.
    let mut document = Document::new();
    let l1 = make_line(&mut document, "(0, 0)", "(1, 0)");
    let l2 = make_line(&mut document, "(0, 1)", "(1, 1)");
    let l3 = make_line(&mut document, "(0, 0)", "(0, 1)");
    assert_message_contains(&mut document, &format!("AreParallel[{l1}, {l2}]"), "true");
    assert_message_contains(&mut document, &format!("AreParallel[{l1}, {l3}]"), "false");
    assert_message_contains(
        &mut document,
        &format!("ArePerpendicular[{l1}, {l3}]"),
        "true",
    );
    assert_message_contains(
        &mut document,
        &format!("ArePerpendicular[{l1}, {l2}]"),
        "false",
    );
    // Concurrentes en origen (puntos medios distintos a propósito).
    let mut document = Document::new();
    let m1 = make_line(&mut document, "(-2, -2)", "(1, 1)");
    let m2 = make_line(&mut document, "(-2, 2)", "(1, -1)");
    let m3 = make_line(&mut document, "(0, -2)", "(0, 2)");
    assert_message_contains(
        &mut document,
        &format!("AreConcurrent[{m1}, {m2}, {m3}]"),
        "true",
    );
    // Concíclicos en círculo unidad.
    let mut document = Document::new();
    let p1 = make_point(&mut document, "(1, 0)");
    let p2 = make_point(&mut document, "(0, 1)");
    let p3 = make_point(&mut document, "(-1, 0)");
    let p4 = make_point(&mut document, "(0, -1)");
    let p5 = make_point(&mut document, "(0, 0)");
    assert_message_contains(
        &mut document,
        &format!("AreConcyclic[{p1}, {p2}, {p3}, {p4}]"),
        "true",
    );
    assert_message_contains(
        &mut document,
        &format!("AreConcyclic[{p1}, {p2}, {p3}, {p5}]"),
        "false",
    );
}

#[test]
fn oleada2_set_caption() {
    let mut document = Document::new();
    let a = make_point(&mut document, "(0, 0)");
    assert_message_contains(
        &mut document,
        &format!("SetCaption[{a}, MiPunto]"),
        "MiPunto",
    );
    // Visible al instante: buscar por rótulo nuevo.
    assert!(
        document.try_find_object_by_label("MiPunto").is_ok(),
        "rótulo nuevo debe ser encontrable"
    );
    // Colisión honesta.
    let b = make_point(&mut document, "(1, 1)");
    match run(&mut document, &format!("SetCaption[{b}, MiPunto]")) {
        CommandOutcome::Error(m) => assert!(m.contains("ya existe"), "{m}"),
        other => panic!("colisión debe dar Error, dio {other:?}"),
    }
}

#[test]
fn oleada2_solidos_platonicos_e_infinitos() {
    let mut document = Document::new();
    assert_ok_or_message(&mut document, "Dodecahedron[0, 0, 0, 2]");
    assert_ok_or_message(&mut document, "Icosahedron[0, 0, 0, 2]");
    assert_ok_or_message(&mut document, "Octahedron[0, 0, 0, 2]");
    assert_ok_or_message(&mut document, "InfiniteCone[0, 0, 0, 0, 1, 0, 30]");
    assert_ok_or_message(&mut document, "InfiniteCylinder[0, 0, 0, 0, 1, 0, 1]");
    assert!(document
        .objects_iter()
        .any(|(_, o)| matches!(o, GeoObject::Platonic3D(p) if p.kind.as_str() == "Dodecahedron")));
    assert!(document
        .objects_iter()
        .any(|(_, o)| matches!(o, GeoObject::Platonic3D(p) if p.kind.as_str() == "Icosahedron")));
    assert!(document
        .objects_iter()
        .any(|(_, o)| matches!(o, GeoObject::Platonic3D(p) if p.kind.as_str() == "Octahedron")));
    assert!(document
        .objects_iter()
        .any(|(_, o)| matches!(o, GeoObject::InfiniteCone3D(_))));
    assert!(document
        .objects_iter()
        .any(|(_, o)| matches!(o, GeoObject::InfiniteCylinder3D(_))));
    // Errores honestos.
    match run(&mut document, "Dodecahedron[0, 0, 0, 0]") {
        CommandOutcome::Error(m) => assert!(m.contains("Dodecahedron"), "{m}"),
        other => panic!("arista 0 debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "InfiniteCone[0, 0, 0, 0, 0, 0, 30]") {
        CommandOutcome::Error(m) => assert!(m.contains("InfiniteCone"), "{m}"),
        other => panic!("dirección nula debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "InfiniteCone[0, 0, 0, 0, 1, 0, 0]") {
        CommandOutcome::Error(m) => assert!(m.contains("InfiniteCone"), "{m}"),
        other => panic!("ángulo 0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn fisica_visual_comandos_crean_objetos() {
    let mut document = Document::new();
    // CampoCargas: 1 carga unitaria en origen → mensaje + VectorField2D con |E|.
    assert_message_contains(&mut document, "CampoCargas[1, 0, 0]", "1 carga");
    let vf = document
        .objects_iter()
        .find_map(|(_, o)| match o {
            GeoObject::VectorField2D(v) => Some(v),
            _ => None,
        })
        .expect("CampoCargas debe crear VectorField2D");
    assert!(vf.color_by_magnitude, "campo E con color=|E|");
    assert!(
        vf.expr_u.contains("sqrt"),
        "Coulomb con 1/r²: {}",
        vf.expr_u
    );
    // E(1,0) de carga unitaria = (1, 0): la expresión sintetizada evalúa.
    let vars = [("x".to_string(), 1.0), ("y".to_string(), 0.0)];
    let ex = grafito_geometry::expr::evaluate(&vf.expr_u, &vars).expect("Ex evalúa");
    let ey = grafito_geometry::expr::evaluate(&vf.expr_v, &vars).expect("Ey evalúa");
    assert!((ex - 1.0).abs() < 1e-9, "Ex(1,0)={ex}");
    assert!(ey.abs() < 1e-9, "Ey(1,0)={ey}");
    // 2 cargas simétricas: mensaje + se anula en el punto medio (0,0).
    assert_message_contains(&mut document, "CampoCargas[1, -1, 0, 1, 1, 0]", "2 carga");
    // Proyectil v0=10, 45°, h0=0 → alcance v0²/g = 10.20 m.
    assert_message_contains(&mut document, "TrayectoriaProyectil[10, 45, 0]", "10.20");
    assert!(document
        .objects_iter()
        .any(|(_, o)| matches!(o, GeoObject::ParametricCurve2D(_))));
    // MAS A=1, ω=2 → T=π=3.14 s.
    assert_message_contains(&mut document, "OsciladorArmonico[1, 2, 0]", "3.14");
    // RC V=5, R=1000, C=0.001 → τ=1 s.
    assert_message_contains(&mut document, "CargaCapacitor[5, 1000, 0.001]", "1.000");
    // Errores honestos.
    match run(&mut document, "TrayectoriaProyectil[-1, 45, 0]") {
        CommandOutcome::Error(m) => assert!(m.contains("TrayectoriaProyectil"), "{m}"),
        other => panic!("v0<0 debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "CargaCapacitor[5, 0, 1]") {
        CommandOutcome::Error(m) => assert!(m.contains("CargaCapacitor"), "{m}"),
        other => panic!("R=0 debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "OsciladorArmonico[1, 0, 0]") {
        CommandOutcome::Error(m) => assert!(m.contains("OsciladorArmonico"), "{m}"),
        other => panic!("omega=0 debe dar Error, dio {other:?}"),
    }
    // Equipotenciales: 1 carga → ImplicitCurve con 8 niveles; V(1,0)=1.
    assert_message_contains(&mut document, "EquipotencialCargas[1, 0, 0]", "8 niveles");
    let ic = document
        .objects_iter()
        .find_map(|(_, o)| match o {
            GeoObject::ImplicitCurve(c) => Some(c),
            _ => None,
        })
        .expect("EquipotencialCargas debe crear ImplicitCurve");
    let levels = ic.contour_levels.as_ref().expect("8 niveles");
    assert_eq!(levels.len(), 8, "niveles: {levels:?}");
    let vars = [("x".to_string(), 1.0), ("y".to_string(), 0.0)];
    let v = grafito_geometry::expr::evaluate(&ic.expr_lhs, &vars).expect("V evalúa");
    assert!((v - 1.0).abs() < 1e-9, "V(1,0)={v}");
    match run(&mut document, "EquipotencialCargas[0, 0, 0]") {
        CommandOutcome::Error(m) => assert!(m.contains("EquipotencialCargas"), "{m}"),
        other => panic!("carga nula debe dar Error, dio {other:?}"),
    }
    // VectoresProyectil: t=0, v0=10, 45° → P=(0,0), |v|=10, crea P+v+a+at+an.
    assert_message_contains(
        &mut document,
        "VectoresProyectil[10, 45, 0, 0]",
        "|v|=10.00",
    );
    let n_lineas = document
        .objects_iter()
        .filter(|(_, o)| matches!(o, GeoObject::Line(_)))
        .count();
    assert_eq!(n_lineas, 4, "v+a+at+an: {n_lineas}");
    // En t=0 con 45°: at=an=g/√2≈6.93 (a=−g ŷ descompuesta en v̂ 45°).
    assert_message_contains(&mut document, "VectoresProyectil[10, 45, 0, 0]", "6.93");
    match run(&mut document, "VectoresProyectil[10, 45, 0, 99]") {
        CommandOutcome::Error(m) => assert!(m.contains("vuelo"), "{m}"),
        other => panic!("t>vuelo debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "VectoresProyectil[10, 45, 0, -1]") {
        CommandOutcome::Error(m) => assert!(m.contains("VectoresProyectil"), "{m}"),
        other => panic!("t<0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn receta_tiro_oblicuo_animado_docs_fisica_animada() {
    // La receta de docs/fisica-animada.md §1+§2 ejecuta sin errores y
    // acumula objetos: 1 paramétrica + 5×(1 punto + 4 flechas) + campo + nivel.
    let mut document = Document::new();
    for cmd in [
        "TrayectoriaProyectil[10, 45, 0]",
        "VectoresProyectil[10, 45, 0, 0]",
        "VectoresProyectil[10, 45, 0, 0.36]",
        "VectoresProyectil[10, 45, 0, 0.72]",
        "VectoresProyectil[10, 45, 0, 1.08]",
        "VectoresProyectil[10, 45, 0, 1.40]",
        "CampoCargas[1, -1, 0, -1, 1, 0]",
        "EquipotencialCargas[1, -1, 0, -1, 1, 0]",
    ] {
        match run(&mut document, cmd) {
            CommandOutcome::Message(_) | CommandOutcome::Ok => {}
            CommandOutcome::Error(m) => panic!("receta {cmd} falló: {m}"),
        }
    }
    let cuenta = |f: fn(&GeoObject) -> bool| document.objects_iter().filter(|(_, o)| f(o)).count();
    assert_eq!(cuenta(|o| matches!(o, GeoObject::ParametricCurve2D(_))), 1);
    assert_eq!(cuenta(|o| matches!(o, GeoObject::Point(_))), 5);
    assert_eq!(cuenta(|o| matches!(o, GeoObject::Line(_))), 20);
    assert_eq!(cuenta(|o| matches!(o, GeoObject::VectorField2D(_))), 1);
    assert_eq!(cuenta(|o| matches!(o, GeoObject::ImplicitCurve(_))), 1);
}

#[test]
fn campo_hilo_biot_savart_azimutal() {
    let mut document = Document::new();
    // I=1 en origen: B(1,0)=(0,1) (antihorario, regla mano derecha).
    assert_message_contains(&mut document, "CampoHilo[1, 0, 0]", "antihorario");
    let vf = document
        .objects_iter()
        .find_map(|(_, o)| match o {
            GeoObject::VectorField2D(v) => Some(v),
            _ => None,
        })
        .expect("CampoHilo debe crear VectorField2D");
    assert!(vf.color_by_magnitude, "campo B con color=|B|");
    let vars = [("x".to_string(), 1.0), ("y".to_string(), 0.0)];
    let bx = grafito_geometry::expr::evaluate(&vf.expr_u, &vars).expect("Bx evalúa");
    let by = grafito_geometry::expr::evaluate(&vf.expr_v, &vars).expect("By evalúa");
    assert!(bx.abs() < 1e-9, "Bx(1,0)={bx}");
    assert!((by - 1.0).abs() < 1e-9, "By(1,0)={by}");
    // I<0 → horario.
    assert_message_contains(&mut document, "CampoHilo[-2, 1, 1]", "horario");
    match run(&mut document, "CampoHilo[0, 0, 0]") {
        CommandOutcome::Error(m) => assert!(m.contains("CampoHilo"), "{m}"),
        other => panic!("I=0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn fem_inducida_faraday_espira_rotante() {
    let mut document = Document::new();
    // N=100, B=0.5, A=0.02, ω=60 → fem₀=100·0.5·0.02·60=60.
    assert_message_contains(&mut document, "FemInducida[100, 0.5, 0.02, 60]", "60.000");
    let f = document
        .objects_iter()
        .find_map(|(_, o)| match o {
            GeoObject::Function(f) => Some(f),
            _ => None,
        })
        .expect("FemInducida debe crear Function");
    assert!(f.expr.contains("sin"), "fem sinusoidal: {}", f.expr);
    let vars = [("x".to_string(), std::f64::consts::FRAC_PI_2 / 60.0)];
    let v = grafito_geometry::expr::evaluate(&f.expr, &vars).expect("fem evalúa");
    assert!((v - 60.0).abs() < 1e-6, "fem(π/2ω)={v}");
    match run(&mut document, "FemInducida[100, 0.5, 0, 60]") {
        CommandOutcome::Error(m) => assert!(m.contains("FemInducida"), "{m}"),
        other => panic!("A=0 debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "FemInducida[100, 0.5, 0.02, 0]") {
        CommandOutcome::Error(m) => assert!(m.contains("FemInducida"), "{m}"),
        other => panic!("omega=0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn barra_hooke_elasticidad_visual() {
    let mut document = Document::new();
    // L0=10, k=50, F=100 → ΔL=2, L=12 (tracción).
    assert_message_contains(&mut document, "BarraHooke[10, 50, 100]", "ΔL=2.000");
    let barras: Vec<_> = document
        .objects_iter()
        .filter_map(|(_, o)| match o {
            GeoObject::Line(l) => Some(l),
            _ => None,
        })
        .collect();
    assert_eq!(barras.len(), 2, "original + deformada");
    let orig = barras.iter().find(|l| l.label == "L0").expect("L0");
    let def = barras.iter().find(|l| l.label == "L").expect("L");
    assert!((orig.end.x - 10.0).abs() < 1e-9, "L0=10");
    assert!((def.end.x - 12.0).abs() < 1e-9, "L=12");
    // Compresión: F<0 acorta y va roja.
    assert_message_contains(&mut document, "BarraHooke[10, 50, -50]", "compresión");
    match run(&mut document, "BarraHooke[10, 50, -600]") {
        CommandOutcome::Error(m) => assert!(m.contains("compresión"), "{m}"),
        other => panic!("L≤0 debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "BarraHooke[10, 0, 100]") {
        CommandOutcome::Error(m) => assert!(m.contains("BarraHooke"), "{m}"),
        other => panic!("k=0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn conduccion_termica_fourier_lineal() {
    let mut document = Document::new();
    // T1=100, T2=0, L=2, k=50 → q=50·100/2=2500, T(x)=100−50x.
    assert_message_contains(
        &mut document,
        "ConduccionTermica[100, 0, 2, 50]",
        "2500.000",
    );
    let f = document
        .objects_iter()
        .find_map(|(_, o)| match o {
            GeoObject::Function(f) => Some(f),
            _ => None,
        })
        .expect("ConduccionTermica debe crear Function");
    for (x, esperado) in [(0.0, 100.0), (1.0, 50.0), (2.0, 0.0)] {
        let vars = [("x".to_string(), x)];
        let v = grafito_geometry::expr::evaluate(&f.expr, &vars).expect("T evalúa");
        assert!((v - esperado).abs() < 1e-9, "T({x})={v}");
    }
    // Equilibrio T1=T2 → q=0.
    assert_message_contains(
        &mut document,
        "ConduccionTermica[20, 20, 1, 5]",
        "equilibrio",
    );
    match run(&mut document, "ConduccionTermica[100, 0, 0, 50]") {
        CommandOutcome::Error(m) => assert!(m.contains("ConduccionTermica"), "{m}"),
        other => panic!("L=0 debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "ConduccionTermica[100, 0, 2, -1]") {
        CommandOutcome::Error(m) => assert!(m.contains("ConduccionTermica"), "{m}"),
        other => panic!("k<0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn gravitacion_campo_atractivo_y_escape() {
    let mut document = Document::new();
    // m=1 en origen: g(1,0)=(−1,0) (atractivo, apunta a la masa).
    assert_message_contains(&mut document, "CampoGravitatorio[1, 0, 0]", "1 masa");
    let vf = document
        .objects_iter()
        .find_map(|(_, o)| match o {
            GeoObject::VectorField2D(v) => Some(v),
            _ => None,
        })
        .expect("CampoGravitatorio debe crear VectorField2D");
    assert!(vf.color_by_magnitude, "campo g con color=|g|");
    let vars = [("x".to_string(), 1.0), ("y".to_string(), 0.0)];
    let gx = grafito_geometry::expr::evaluate(&vf.expr_u, &vars).expect("gx evalúa");
    let gy = grafito_geometry::expr::evaluate(&vf.expr_v, &vars).expect("gy evalúa");
    assert!((gx + 1.0).abs() < 1e-9, "gx(1,0)={gx}");
    assert!(gy.abs() < 1e-9, "gy(1,0)={gy}");
    // 2 masas iguales simétricas: se anula en el punto medio.
    assert_message_contains(
        &mut document,
        "CampoGravitatorio[1, -1, 0, 1, 1, 0]",
        "2 masa",
    );
    // Tierra: M=5.972e24, R=6.371e6 → 11186 m/s.
    assert_message_contains(
        &mut document,
        "VelocidadEscape[5.972e24, 6.371e6]",
        "11186.0",
    );
    match run(&mut document, "CampoGravitatorio[0, 0, 0]") {
        CommandOutcome::Error(m) => assert!(m.contains("CampoGravitatorio"), "{m}"),
        other => panic!("m=0 debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "VelocidadEscape[1, 0]") {
        CommandOutcome::Error(m) => assert!(m.contains("VelocidadEscape"), "{m}"),
        other => panic!("R=0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn tubo_venturi_continuidad_y_bernoulli() {
    let mut document = Document::new();
    // A1=4, A2=1, v1=1, p1=1e5, rho=1000 → v2=4, p2=1e5+500(1−16)=92500.
    assert_message_contains(
        &mut document,
        "TuboVenturi[4, 1, 1, 100000, 1000]",
        "v2=4.000",
    );
    assert_message_contains(
        &mut document,
        "TuboVenturi[4, 1, 1, 100000, 1000]",
        "92500.0",
    );
    let lineas: Vec<_> = document
        .objects_iter()
        .filter_map(|(_, o)| match o {
            GeoObject::Line(l) => Some(l),
            _ => None,
        })
        .collect();
    assert_eq!(lineas.len(), 8, "2 tubos + 2 flechas ×2 corridas");
    // La flecha v2 es más larga que v1 (continuidad a la vista).
    let f1 = lineas.iter().find(|l| l.label == "v1").expect("v1");
    let f2 = lineas.iter().find(|l| l.label == "v2").expect("v2");
    let len = |l: &LineObj| ((l.end.x - l.start.x).powi(2) + (l.end.y - l.start.y).powi(2)).sqrt();
    assert!(len(f2) > len(f1), "v2>v1 dibujado");
    match run(&mut document, "TuboVenturi[0, 1, 1, 100000, 1000]") {
        CommandOutcome::Error(m) => assert!(m.contains("TuboVenturi"), "{m}"),
        other => panic!("A1=0 debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "TuboVenturi[4, 1, -1, 100000, 1000]") {
        CommandOutcome::Error(m) => assert!(m.contains("TuboVenturi"), "{m}"),
        other => panic!("v1<0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn rayo_refraccion_snell_y_tir() {
    let mut document = Document::new();
    // Aire→vidrio, θ1=30°: sinθ2=sin30/1.5=1/3 → θ2=19.47°.
    assert_message_contains(&mut document, "RayoRefraccion[1, 1.5, 30]", "19.47");
    let n1 = document
        .objects_iter()
        .filter(|(_, o)| matches!(o, GeoObject::Line(_)))
        .count();
    assert_eq!(n1, 3, "interfaz + incidente + refractado");
    // El refractado (t) se acerca a la normal: |x| menor que el incidente.
    let refr = document
        .objects_iter()
        .find_map(|(_, o)| match o {
            GeoObject::Line(l) if l.label == "t" => Some(l),
            _ => None,
        })
        .expect("rayo refractado t");
    assert!((refr.end.x - 5.0 * (1.0 / 3.0_f64).asin().sin()).abs() < 1e-9);
    // Vidrio→aire, θ1=60° > θc=41.8°: TIR con reflejado.
    assert_message_contains(&mut document, "RayoRefraccion[1.5, 1, 60]", "TIR");
    assert!(document.objects_iter().any(|(_, o)| match o {
        GeoObject::Line(l) => l.label == "r",
        _ => false,
    }));
    match run(&mut document, "RayoRefraccion[1, 1.5, 90]") {
        CommandOutcome::Error(m) => assert!(m.contains("RayoRefraccion"), "{m}"),
        other => panic!("θ1=90 debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "RayoRefraccion[0, 1, 30]") {
        CommandOutcome::Error(m) => assert!(m.contains("RayoRefraccion"), "{m}"),
        other => panic!("n1=0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn onda_estacionaria_envolvente_y_nodos() {
    let mut document = Document::new();
    // L=2, n=3, A=1: λ=4/3, 4 nodos en x=0, 2/3, 4/3, 2.
    assert_message_contains(&mut document, "OndaEstacionaria[2, 3, 1]", "1.333");
    assert_message_contains(&mut document, "OndaEstacionaria[2, 3, 1]", "4 nodos");
    let f = document
        .objects_iter()
        .find_map(|(_, o)| match o {
            GeoObject::Function(f) => Some(f),
            _ => None,
        })
        .expect("OndaEstacionaria debe crear Function");
    assert!(f.expr.contains("sin"), "envolvente sinusoidal: {}", f.expr);
    // La envolvente vale 0 en los nodos y ±2 en el vientre x=L/2n.
    for x in [0.0, 2.0 / 3.0, 4.0 / 3.0, 2.0] {
        let vars = [("x".to_string(), x)];
        let v = grafito_geometry::expr::evaluate(&f.expr, &vars).expect("y evalúa");
        assert!(v.abs() < 1e-9, "nodo x={x}: y={v}");
    }
    let vars = [("x".to_string(), 2.0 / 6.0)];
    let v = grafito_geometry::expr::evaluate(&f.expr, &vars).expect("vientre");
    assert!((v - 2.0).abs() < 1e-9, "vientre y={v}");
    match run(&mut document, "OndaEstacionaria[2, 2.5, 1]") {
        CommandOutcome::Error(m) => assert!(m.contains("OndaEstacionaria"), "{m}"),
        other => panic!("n no entero debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "OndaEstacionaria[0, 3, 1]") {
        CommandOutcome::Error(m) => assert!(m.contains("OndaEstacionaria"), "{m}"),
        other => panic!("L=0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn gas_ideal_isoterma_boyle() {
    let mut document = Document::new();
    // n=1, T=300: nRT=2494.2, p(1)=2494.2 Pa.
    assert_message_contains(&mut document, "GasIdeal[1, 300]", "2494.2");
    let f = document
        .objects_iter()
        .find_map(|(_, o)| match o {
            GeoObject::Function(f) => Some(f),
            _ => None,
        })
        .expect("GasIdeal debe crear Function");
    for (v, p) in [(1.0, 2494.2), (2.0, 1247.1)] {
        let vars = [("x".to_string(), v)];
        let got = grafito_geometry::expr::evaluate(&f.expr, &vars).expect("p evalúa");
        assert!((got - p).abs() < 0.05, "p({v})={got}");
    }
    match run(&mut document, "GasIdeal[1, 0]") {
        CommandOutcome::Error(m) => assert!(m.contains("GasIdeal"), "{m}"),
        other => panic!("T=0 debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "GasIdeal[-1, 300]") {
        CommandOutcome::Error(m) => assert!(m.contains("GasIdeal"), "{m}"),
        other => panic!("n<0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn resonancia_rlc_curva_y_f0_q() {
    let mut document = Document::new();
    // R=10, L=0.1, C=1e-4: f₀=1/2π√(1e-5)=50.33 Hz, Q=√(1000)/10=3.16.
    assert_message_contains(&mut document, "ResonanciaRLC[10, 0.1, 0.0001]", "50.33");
    assert_message_contains(&mut document, "ResonanciaRLC[10, 0.1, 0.0001]", "Q=3.16");
    let f = document
        .objects_iter()
        .find_map(|(_, o)| match o {
            GeoObject::Function(f) => Some(f),
            _ => None,
        })
        .expect("ResonanciaRLC debe crear Function");
    // En resonancia la impedancia es R: I(f₀)=1/10=0.1.
    let vars = [("x".to_string(), 50.33)];
    let got = grafito_geometry::expr::evaluate(&f.expr, &vars).expect("I evalúa");
    assert!((got - 0.1).abs() < 0.001, "I(f₀)={got}");
    match run(&mut document, "ResonanciaRLC[0, 0.1, 0.0001]") {
        CommandOutcome::Error(m) => assert!(m.contains("ResonanciaRLC"), "{m}"),
        other => panic!("R=0 debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "ResonanciaRLC[10, 0.1, 0]") {
        CommandOutcome::Error(m) => assert!(m.contains("ResonanciaRLC"), "{m}"),
        other => panic!("C=0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn efecto_doppler_frentes_y_frecuencias() {
    let mut document = Document::new();
    // v=100, c=340, f=440: f'=440·340/240=623.3, f''=440·340/440=340.
    assert_message_contains(&mut document, "EfectoDoppler[100, 340, 440]", "623.3");
    assert_message_contains(&mut document, "EfectoDoppler[100, 340, 440]", "340.0");
    let circulos: Vec<_> = document
        .objects_iter()
        .filter_map(|(_, o)| match o {
            GeoObject::Circle(c) => Some(c),
            _ => None,
        })
        .collect();
    assert_eq!(circulos.len(), 12, "6 frentes ×2 corridas");
    // Los frentes viejos son más grandes y nacen más atrás:
    // ordenando por centro (más negativo primero), los radios decrecen.
    let mut pares: Vec<(f64, f64)> = circulos.iter().map(|c| (c.center.x, c.radius)).collect();
    pares.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("finitos"));
    assert!(
        pares.windows(2).all(|w| w[0].1 >= w[1].1),
        "radios decrecen hacia +x: {pares:?}"
    );
    assert!(
        (pares[0].1 - 6.0 * 340.0 / 440.0).abs() < 1e-9,
        "radio mayor"
    );
    match run(&mut document, "EfectoDoppler[340, 340, 440]") {
        CommandOutcome::Error(m) => assert!(m.contains("EfectoDoppler"), "{m}"),
        other => panic!("v=c debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "EfectoDoppler[-1, 340, 440]") {
        CommandOutcome::Error(m) => assert!(m.contains("EfectoDoppler"), "{m}"),
        other => panic!("v<0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn doble_rendija_franjas_young() {
    let mut document = Document::new();
    // λ=5e-7, d=1e-4, L=1: Δy=5e-3.
    assert_message_contains(
        &mut document,
        "DobleRendija[0.0000005, 0.0001, 1]",
        "0.0050",
    );
    let f = document
        .objects_iter()
        .find_map(|(_, o)| match o {
            GeoObject::Function(f) => Some(f),
            _ => None,
        })
        .expect("DobleRendija debe crear Function");
    // Máximos en y=k·Δy, mínimos a mitad de camino.
    for (y, esperado) in [(0.0, 1.0), (0.005, 1.0), (0.0025, 0.0), (-0.005, 1.0)] {
        let vars = [("x".to_string(), y)];
        let got = grafito_geometry::expr::evaluate(&f.expr, &vars).expect("I evalúa");
        assert!((got - esperado).abs() < 1e-9, "I({y})={got}");
    }
    match run(&mut document, "DobleRendija[0, 0.0001, 1]") {
        CommandOutcome::Error(m) => assert!(m.contains("DobleRendija"), "{m}"),
        other => panic!("λ=0 debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "DobleRendija[0.0000005, -1, 1]") {
        CommandOutcome::Error(m) => assert!(m.contains("DobleRendija"), "{m}"),
        other => panic!("d<0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn diagrama_minkowski_cono_y_gamma() {
    let mut document = Document::new();
    // v=0.6c: β=0.6, γ=1.25.
    assert_message_contains(&mut document, "DiagramaMinkowski[0.6, 1]", "1.250");
    let lineas: Vec<_> = document
        .objects_iter()
        .filter_map(|(_, o)| match o {
            GeoObject::Line(l) => Some(l),
            _ => None,
        })
        .collect();
    assert_eq!(lineas.len(), 3, "2 luz + 1 observador");
    // El observador a 0.6c va de (−3,−5) a (3,5): pendiente 5/3=c/v.
    let obs = lineas.iter().find(|l| l.label == "v").expect("observador");
    assert!((obs.start.x + 3.0).abs() < 1e-9, "x0={}", obs.start.x);
    assert!((obs.end.x - 3.0).abs() < 1e-9, "x1={}", obs.end.x);
    match run(&mut document, "DiagramaMinkowski[1, 1]") {
        CommandOutcome::Error(m) => assert!(m.contains("DiagramaMinkowski"), "{m}"),
        other => panic!("v=c debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "DiagramaMinkowski[-1, 1]") {
        CommandOutcome::Error(m) => assert!(m.contains("DiagramaMinkowski"), "{m}"),
        other => panic!("v<0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn desintegracion_exponencial_y_vida_media() {
    let mut document = Document::new();
    // N0=1000, λ=0.1: t½=6.931, N(t½)=500, N(0)=1000.
    assert_message_contains(&mut document, "Desintegracion[1000, 0.1]", "6.931");
    let f = document
        .objects_iter()
        .find_map(|(_, o)| match o {
            GeoObject::Function(f) => Some(f),
            _ => None,
        })
        .expect("Desintegracion debe crear Function");
    for (t, esperado) in [(0.0, 1000.0), (6.931471805599453, 500.0)] {
        let vars = [("x".to_string(), t)];
        let got = grafito_geometry::expr::evaluate(&f.expr, &vars).expect("N evalúa");
        assert!((got - esperado).abs() < 0.01, "N({t})={got}");
    }
    match run(&mut document, "Desintegracion[1000, 0]") {
        CommandOutcome::Error(m) => assert!(m.contains("Desintegracion"), "{m}"),
        other => panic!("λ=0 debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "Desintegracion[-5, 0.1]") {
        CommandOutcome::Error(m) => assert!(m.contains("Desintegracion"), "{m}"),
        other => panic!("N0<0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn efecto_fotoelectrico_umbral_y_kmax() {
    let mut document = Document::new();
    // Luz verde 5.66e14 Hz sobre cesio W=3.43e-19 J:
    // hf=3.75e-19, K=3.2e-20 J = 0.20 eV.
    assert_message_contains(
        &mut document,
        "EfectoFotoelectrico[5.66e14, 3.43e-19]",
        "0.20 eV",
    );
    // Bajo el umbral: error honesto, no K negativa.
    match run(&mut document, "EfectoFotoelectrico[1e14, 3.43e-19]") {
        CommandOutcome::Error(m) => assert!(m.contains("sin emisión"), "{m}"),
        other => panic!("hf<W debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "EfectoFotoelectrico[0, 3.43e-19]") {
        CommandOutcome::Error(m) => assert!(m.contains("EfectoFotoelectrico"), "{m}"),
        other => panic!("f=0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn pendulo_dibujo_y_periodo_corregido() {
    let mut document = Document::new();
    // L=1, θ₀=30°: T₀=2.007, corr=1+0.0171+0.0003 → T=2.042 (+1.7%).
    assert_message_contains(&mut document, "Pendulo[1, 30]", "2.042");
    let hilos: Vec<_> = document
        .objects_iter()
        .filter_map(|(_, o)| match o {
            GeoObject::Line(l) if l.label == "L" => Some(l),
            _ => None,
        })
        .collect();
    assert_eq!(hilos.len(), 1, "un hilo");
    let h = hilos[0];
    assert!((h.end.x - 0.5).abs() < 1e-9, "bx=sin30={}", h.end.x);
    assert!(
        (h.end.y + 0.8660254037844386).abs() < 1e-9,
        "by={}",
        h.end.y
    );
    assert!(
        document
            .objects_iter()
            .any(|(_, o)| matches!(o, GeoObject::Circle(_))),
        "lenteja"
    );
    match run(&mut document, "Pendulo[1, 90]") {
        CommandOutcome::Error(m) => assert!(m.contains("Pendulo"), "{m}"),
        other => panic!("θ₀>60 debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "Pendulo[0, 30]") {
        CommandOutcome::Error(m) => assert!(m.contains("Pendulo"), "{m}"),
        other => panic!("L=0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn ciclo_carnot_banda_y_rendimiento() {
    let mut document = Document::new();
    // Tc=300, Th=600: η=50%.
    assert_message_contains(&mut document, "CicloCarnot[300, 600]", "50.0%");
    let isotermas: Vec<_> = document
        .objects_iter()
        .filter_map(|(_, o)| match o {
            GeoObject::Function(f) => Some(f),
            _ => None,
        })
        .collect();
    assert_eq!(isotermas.len(), 2, "Tc + Th");
    // La caliente está por encima: p(1)=R·T (ordenar, iteración inversa).
    let vars = [("x".to_string(), 1.0)];
    let mut ps: Vec<f64> = isotermas
        .iter()
        .map(|f| grafito_geometry::expr::evaluate(&f.expr, &vars).expect("p"))
        .collect();
    ps.sort_by(|a, b| a.partial_cmp(b).expect("finitos"));
    assert!((ps[0] - 8.314 * 300.0).abs() < 0.01, "p_c={}", ps[0]);
    assert!((ps[1] - 8.314 * 600.0).abs() < 0.01, "p_h={}", ps[1]);
    match run(&mut document, "CicloCarnot[600, 300]") {
        CommandOutcome::Error(m) => assert!(m.contains("CicloCarnot"), "{m}"),
        other => panic!("Tc≥Th debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "CicloCarnot[0, 600]") {
        CommandOutcome::Error(m) => assert!(m.contains("CicloCarnot"), "{m}"),
        other => panic!("Tc=0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn lente_delgada_rayos_e_imagen() {
    let mut document = Document::new();
    // f=2, do=6, ho=1: di=3, hi=−0.5, M=−0.5.
    assert_message_contains(&mut document, "LenteDelgada[2, 6, 1]", "di=3.000");
    assert_message_contains(&mut document, "LenteDelgada[2, 6, 1]", "hi=-0.500");
    let rayos: Vec<_> = document
        .objects_iter()
        .filter_map(|(_, o)| match o {
            GeoObject::Line(l) if l.label.starts_with("rayo") => Some(l),
            _ => None,
        })
        .collect();
    assert_eq!(rayos.len(), 6, "3 rayos ×2 corridas");
    // La imagen dibujada está en x=+3, y=−0.5.
    let img = document
        .objects_iter()
        .find_map(|(_, o)| match o {
            GeoObject::Line(l) if l.label.starts_with("imagen") => Some(l),
            _ => None,
        })
        .expect("imagen");
    assert!((img.start.x - 3.0).abs() < 1e-9, "x={}", img.start.x);
    assert!((img.end.y + 0.5).abs() < 1e-9, "y={}", img.end.y);
    match run(&mut document, "LenteDelgada[2, 2, 1]") {
        CommandOutcome::Error(m) => assert!(m.contains("LenteDelgada"), "{m}"),
        other => panic!("do=f debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "LenteDelgada[2, 1, 1]") {
        CommandOutcome::Error(m) => assert!(m.contains("LenteDelgada"), "{m}"),
        other => panic!("do<f debe dar Error, dio {other:?}"),
    }
}

#[test]
fn caida_rozamiento_terminal_stokes() {
    let mut document = Document::new();
    // m=1, c=0.5, v0=0: vt=19.6, v(0)=0, v(∞)→vt.
    assert_message_contains(&mut document, "CaidaRozamiento[1, 0.5, 0]", "19.60");
    let f = document
        .objects_iter()
        .find_map(|(_, o)| match o {
            GeoObject::Function(f) => Some(f),
            _ => None,
        })
        .expect("CaidaRozamiento debe crear Function");
    for (t, esperado) in [(0.0, 0.0), (100.0, 19.6)] {
        let vars = [("x".to_string(), t)];
        let got = grafito_geometry::expr::evaluate(&f.expr, &vars).expect("v evalúa");
        assert!((got - esperado).abs() < 0.01, "v({t})={got}");
    }
    match run(&mut document, "CaidaRozamiento[1, 0, 0]") {
        CommandOutcome::Error(m) => assert!(m.contains("CaidaRozamiento"), "{m}"),
        other => panic!("c=0 debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "CaidaRozamiento[-1, 0.5, 0]") {
        CommandOutcome::Error(m) => assert!(m.contains("CaidaRozamiento"), "{m}"),
        other => panic!("m<0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn efecto_compton_desplazamiento() {
    let mut document = Document::new();
    // θ=90°: Δλ=λc=2.426e-12; θ=180°: Δλ=2λc=4.853e-12.
    assert_message_contains(&mut document, "EfectoCompton[90]", "2.426e-12");
    assert_message_contains(&mut document, "EfectoCompton[180]", "4.853e-12");
    assert_message_contains(&mut document, "EfectoCompton[0]", "0.000e0");
    match run(&mut document, "EfectoCompton[200]") {
        CommandOutcome::Error(m) => assert!(m.contains("EfectoCompton"), "{m}"),
        other => panic!("θ>180 debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "EfectoCompton[-5]") {
        CommandOutcome::Error(m) => assert!(m.contains("EfectoCompton"), "{m}"),
        other => panic!("θ<0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn potencial_gravitatorio_pozos_negativos() {
    let mut document = Document::new();
    assert_message_contains(&mut document, "PotencialGravitatorio[1, 0, 0]", "V<0");
    let curvas: Vec<_> = document
        .objects_iter()
        .filter_map(|(_, o)| match o {
            GeoObject::ImplicitCurve(c) => Some(c),
            _ => None,
        })
        .collect();
    assert_eq!(curvas.len(), 1, "1 pozo ×1 corrida");
    // V(1,0)=−1 y niveles [−0.25,−0.5,−1,−2].
    let vars = [("x".to_string(), 1.0), ("y".to_string(), 0.0)];
    let got = grafito_geometry::expr::evaluate(&curvas[0].expr_lhs, &vars).expect("V evalúa");
    assert!((got + 1.0).abs() < 1e-9, "V(1,0)={got}");
    assert_eq!(
        curvas[0].contour_levels,
        Some(vec![-0.25, -0.5, -1.0, -2.0]),
        "niveles"
    );
    match run(&mut document, "PotencialGravitatorio[-1, 0, 0]") {
        CommandOutcome::Error(m) => assert!(m.contains("PotencialGravitatorio"), "{m}"),
        other => panic!("m<0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn espejo_esferico_imagen_mismo_lado() {
    let mut document = Document::new();
    // f=2, do=6, ho=1: di=3, hi=−0.5 (imagen en x=−3).
    assert_message_contains(&mut document, "EspejoEsferico[2, 6, 1]", "di=3.000");
    let img = document
        .objects_iter()
        .find_map(|(_, o)| match o {
            GeoObject::Line(l) if l.label.starts_with("imagen") => Some(l),
            _ => None,
        })
        .expect("imagen");
    assert!((img.start.x + 3.0).abs() < 1e-9, "x={}", img.start.x);
    assert!((img.end.y + 0.5).abs() < 1e-9, "y={}", img.end.y);
    match run(&mut document, "EspejoEsferico[2, 2, 1]") {
        CommandOutcome::Error(m) => assert!(m.contains("EspejoEsferico"), "{m}"),
        other => panic!("do=f debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "EspejoEsferico[2, 1, 1]") {
        CommandOutcome::Error(m) => assert!(m.contains("EspejoEsferico"), "{m}"),
        other => panic!("do<f debe dar Error, dio {other:?}"),
    }
}

#[test]
fn ley_malus_cos2_y_extincion() {
    let mut document = Document::new();
    // I0=100, θ=60°: I=25. A 90°: 0.
    assert_message_contains(&mut document, "LeyMalus[100, 60]", "25.000");
    assert_message_contains(&mut document, "LeyMalus[100, 90]", "0.000");
    let f = document
        .objects_iter()
        .find_map(|(_, o)| match o {
            GeoObject::Function(f) => Some(f),
            _ => None,
        })
        .expect("LeyMalus debe crear Function");
    // La curva vive en radianes: I(0)=100, I(π/2)=0.
    for (x, esperado) in [(0.0, 100.0), (std::f64::consts::FRAC_PI_2, 0.0)] {
        let vars = [("x".to_string(), x)];
        let got = grafito_geometry::expr::evaluate(&f.expr, &vars).expect("I evalúa");
        assert!((got - esperado).abs() < 1e-9, "I({x})={got}");
    }
    match run(&mut document, "LeyMalus[0, 60]") {
        CommandOutcome::Error(m) => assert!(m.contains("LeyMalus"), "{m}"),
        other => panic!("I0=0 debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "LeyMalus[-5, 60]") {
        CommandOutcome::Error(m) => assert!(m.contains("LeyMalus"), "{m}"),
        other => panic!("I0<0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn ley_wien_pico_cuerpo_negro() {
    let mut document = Document::new();
    // Sol 5800 K: λ_max=4.996e-7 m (verde). Cuerpo 300 K: 9.659e-6.
    assert_message_contains(&mut document, "LeyWien[5800]", "4.996e-7");
    assert_message_contains(&mut document, "LeyWien[300]", "9.659e-6");
    match run(&mut document, "LeyWien[0]") {
        CommandOutcome::Error(m) => assert!(m.contains("LeyWien"), "{m}"),
        other => panic!("T=0 debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "LeyWien[-10]") {
        CommandOutcome::Error(m) => assert!(m.contains("LeyWien"), "{m}"),
        other => panic!("T<0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn potencia_stefan_sigma_at4() {
    let mut document = Document::new();
    // Sol: σ·6.09e18·5800⁴ → 3.908e26 W.
    assert_message_contains(&mut document, "PotenciaStefan[5800, 6.09e18]", "3.908e26");
    // Cuerpo 1 m² a 300 K: σ·8100000000=459.3 W.
    assert_message_contains(&mut document, "PotenciaStefan[300, 1]", "4.593e2");
    match run(&mut document, "PotenciaStefan[300, 0]") {
        CommandOutcome::Error(m) => assert!(m.contains("PotenciaStefan"), "{m}"),
        other => panic!("A=0 debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "PotenciaStefan[-5, 1]") {
        CommandOutcome::Error(m) => assert!(m.contains("PotenciaStefan"), "{m}"),
        other => panic!("T<0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn presion_hidrostatica_rho_g_h() {
    let mut document = Document::new();
    // Agua 10 m: 98000 Pa = 0.97 atm.
    assert_message_contains(&mut document, "PresionHidrostatica[1000, 10]", "98000 Pa");
    assert_message_contains(&mut document, "PresionHidrostatica[1000, 10]", "0.97 atm");
    // Superficie: 0 Pa.
    assert_message_contains(&mut document, "PresionHidrostatica[1000, 0]", "0 Pa");
    match run(&mut document, "PresionHidrostatica[0, 10]") {
        CommandOutcome::Error(m) => assert!(m.contains("PresionHidrostatica"), "{m}"),
        other => panic!("rho=0 debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "PresionHidrostatica[1000, -2]") {
        CommandOutcome::Error(m) => assert!(m.contains("PresionHidrostatica"), "{m}"),
        other => panic!("h<0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn empuje_arquimedes_flota_o_se_hunde() {
    let mut document = Document::new();
    // Hielo (917 kg/m³, 1 m³, 917 kg): E=peso → FLOTA (límite).
    assert_message_contains(&mut document, "EmpujeArquimedes[1000, 1, 917]", "FLOTA");
    // Hierro (1 m³, 7800 kg): 9800 N vs 76440 N → SE HUNDE.
    assert_message_contains(&mut document, "EmpujeArquimedes[1000, 1, 7800]", "SE HUNDE");
    assert_message_contains(&mut document, "EmpujeArquimedes[1000, 1, 7800]", "9800.0 N");
    match run(&mut document, "EmpujeArquimedes[0, 1, 5]") {
        CommandOutcome::Error(m) => assert!(m.contains("EmpujeArquimedes"), "{m}"),
        other => panic!("rho=0 debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "EmpujeArquimedes[1000, -1, 5]") {
        CommandOutcome::Error(m) => assert!(m.contains("EmpujeArquimedes"), "{m}"),
        other => panic!("V<0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn numero_reynolds_regimen() {
    let mut document = Document::new();
    // Agua en caño 1 cm a 0.1 m/s: Re=1000 → LAMINAR.
    assert_message_contains(
        &mut document,
        "NumeroReynolds[1000, 0.1, 0.01, 0.001]",
        "LAMINAR",
    );
    // La misma a 5 m/s: Re=50000 → TURBULENTO.
    assert_message_contains(
        &mut document,
        "NumeroReynolds[1000, 5, 0.01, 0.001]",
        "TURBULENTO",
    );
    // Transición: Re=3000.
    assert_message_contains(
        &mut document,
        "NumeroReynolds[1000, 3, 0.001, 0.001]",
        "TRANSICIÓN",
    );
    match run(&mut document, "NumeroReynolds[1000, 1, 0.01, 0]") {
        CommandOutcome::Error(m) => assert!(m.contains("NumeroReynolds"), "{m}"),
        other => panic!("μ=0 debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "NumeroReynolds[1000, -1, 0.01, 0.001]") {
        CommandOutcome::Error(m) => assert!(m.contains("NumeroReynolds"), "{m}"),
        other => panic!("v<0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn dilatacion_lineal_calor_alarga_frio_acorta() {
    let mut document = Document::new();
    // Acero 100 m, α=1.2e-5, ΔT=40 K: ΔL=0.048 m → L=100.048.
    assert_message_contains(
        &mut document,
        "DilatacionLineal[100, 0.000012, 40]",
        "4.800e-2",
    );
    assert_message_contains(
        &mut document,
        "DilatacionLineal[100, 0.000012, 40]",
        "100.048",
    );
    // Frío: ΔT negativo acorta (contracción honesta, no error).
    assert_message_contains(
        &mut document,
        "DilatacionLineal[100, 0.000012, -40]",
        "99.952",
    );
    match run(&mut document, "DilatacionLineal[0, 0.000012, 40]") {
        CommandOutcome::Error(m) => assert!(m.contains("DilatacionLineal"), "{m}"),
        other => panic!("L0=0 debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "DilatacionLineal[100, -0.000012, 40]") {
        CommandOutcome::Error(m) => assert!(m.contains("DilatacionLineal"), "{m}"),
        other => panic!("α<0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn plano_inclinado_cuna_y_fuerzas() {
    let mut document = Document::new();
    // m=2, θ=30°: a=4.900, N=16.97. 3 cuña + 3 flechas + 1 bloque.
    let antes = document.objects_iter().count();
    assert_message_contains(&mut document, "PlanoInclinado[2, 30]", "4.900");
    assert_message_contains(&mut document, "PlanoInclinado[2, 30]", "16.97");
    let lineas = document
        .objects_iter()
        .filter(|(_, o)| matches!(o, GeoObject::Line(_)))
        .count();
    assert_eq!(
        lineas, 12,
        "3 cuña + 3 flechas por cada una de las 2 corridas"
    );
    assert_eq!(
        document.objects_iter().count() - antes,
        14,
        "6 líneas + 1 punto por corrida"
    );
    match run(&mut document, "PlanoInclinado[2, 90]") {
        CommandOutcome::Error(m) => assert!(m.contains("PlanoInclinado"), "{m}"),
        other => panic!("θ=90 debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "PlanoInclinado[0, 30]") {
        CommandOutcome::Error(m) => assert!(m.contains("PlanoInclinado"), "{m}"),
        other => panic!("m=0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn ley_ohm_corriente_y_potencia() {
    let mut document = Document::new();
    // 12 V, 4 Ω: I=3 A, P=36 W.
    assert_message_contains(&mut document, "LeyOhm[12, 4]", "3.000 A");
    assert_message_contains(&mut document, "LeyOhm[12, 4]", "36.000 W");
    // LED: 2 V, 220 Ω → 9.091 mA.
    assert_message_contains(&mut document, "LeyOhm[2, 220]", "0.009 A");
    match run(&mut document, "LeyOhm[12, 0]") {
        CommandOutcome::Error(m) => assert!(m.contains("LeyOhm"), "{m}"),
        other => panic!("R=0 debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "LeyOhm[12, -4]") {
        CommandOutcome::Error(m) => assert!(m.contains("LeyOhm"), "{m}"),
        other => panic!("R<0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn calor_especifico_pava_electrica() {
    let mut document = Document::new();
    // 1 kg agua (4186), ΔT=80 K: Q=334880 J.
    assert_message_contains(
        &mut document,
        "CalorEspecifico[1, 4186, 80]",
        "334880.000 J",
    );
    // Enfriar: ΔT negativo cede calor (no error).
    assert_message_contains(
        &mut document,
        "CalorEspecifico[1, 4186, -80]",
        "-334880.000 J",
    );
    match run(&mut document, "CalorEspecifico[0, 4186, 80]") {
        CommandOutcome::Error(m) => assert!(m.contains("CalorEspecifico"), "{m}"),
        other => panic!("m=0 debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "CalorEspecifico[1, 0, 80]") {
        CommandOutcome::Error(m) => assert!(m.contains("CalorEspecifico"), "{m}"),
        other => panic!("c=0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn fuerza_coulomb_signo_decide() {
    let mut document = Document::new();
    // 1 µC × 1 µC a 1 m: F=8.988 mN repulsión.
    assert_message_contains(
        &mut document,
        "FuerzaCoulomb[0.000001, 0.000001, 1]",
        "8.988e-3",
    );
    assert_message_contains(
        &mut document,
        "FuerzaCoulomb[0.000001, 0.000001, 1]",
        "REPULSIÓN",
    );
    // Signos opuestos: atracción.
    assert_message_contains(
        &mut document,
        "FuerzaCoulomb[0.000001, -0.000001, 1]",
        "ATRACCIÓN",
    );
    match run(&mut document, "FuerzaCoulomb[0.000001, 0.000001, 0]") {
        CommandOutcome::Error(m) => assert!(m.contains("FuerzaCoulomb"), "{m}"),
        other => panic!("r=0 debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "FuerzaCoulomb[0.000001, 0.000001, -1]") {
        CommandOutcome::Error(m) => assert!(m.contains("FuerzaCoulomb"), "{m}"),
        other => panic!("r<0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn ley_enfriamiento_misma_edo_que_rc() {
    let mut document = Document::new();
    // Café 90°C, ambiente 20°C, k=0.1: τ=10 s.
    assert_message_contains(&mut document, "LeyEnfriamiento[90, 20, 0.1]", "20°C");
    assert_message_contains(&mut document, "LeyEnfriamiento[90, 20, 0.1]", "10.000 s");
    // La curva vive: hay una Function con la exponencial.
    let vivas = document
        .objects_iter()
        .filter(|(_, o)| matches!(o, GeoObject::Function(_)))
        .count();
    assert_eq!(vivas, 2, "una Function por cada corrida");
    match run(&mut document, "LeyEnfriamiento[90, 20, 0]") {
        CommandOutcome::Error(m) => assert!(m.contains("LeyEnfriamiento"), "{m}"),
        other => panic!("k=0 debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "LeyEnfriamiento[90, 20, -0.1]") {
        CommandOutcome::Error(m) => assert!(m.contains("LeyEnfriamiento"), "{m}"),
        other => panic!("k<0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn onda_de_broglie_electron_vs_persona() {
    let mut document = Document::new();
    // Electrón (9.11e-31 kg) a 1e6 m/s: λ=7.273e-10 m.
    assert_message_contains(
        &mut document,
        "OndaDeBroglie[9.11e-31, 1000000]",
        "7.273e-10",
    );
    // Persona 70 kg a 1 m/s: λ=9.466e-36 m (clásica, aceptalo).
    assert_message_contains(&mut document, "OndaDeBroglie[70, 1]", "9.466e-36");
    match run(&mut document, "OndaDeBroglie[70, 0]") {
        CommandOutcome::Error(m) => assert!(m.contains("OndaDeBroglie"), "{m}"),
        other => panic!("v=0 debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "OndaDeBroglie[-1, 1]") {
        CommandOutcome::Error(m) => assert!(m.contains("OndaDeBroglie"), "{m}"),
        other => panic!("m<0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn energia_mecanica_se_conserva() {
    let mut document = Document::new();
    // 2 kg, 3 m/s, 5 m: Ec=9, Ep=98, Em=107.
    assert_message_contains(&mut document, "EnergiaMecanica[2, 3, 5]", "Ec=9.000 J");
    assert_message_contains(&mut document, "EnergiaMecanica[2, 3, 5]", "Ep=98.000 J");
    assert_message_contains(&mut document, "EnergiaMecanica[2, 3, 5]", "Em=107.000 J");
    // v con signo: la energía usa v².
    assert_message_contains(&mut document, "EnergiaMecanica[2, -3, 5]", "Em=107.000 J");
    match run(&mut document, "EnergiaMecanica[0, 3, 5]") {
        CommandOutcome::Error(m) => assert!(m.contains("EnergiaMecanica"), "{m}"),
        other => panic!("m=0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn fuerza_lorentz_mano_derecha() {
    let mut document = Document::new();
    // Protón (1.6e-19 C) a 1e6 m/s en 1 T: F=1.6e-13 N.
    assert_message_contains(
        &mut document,
        "FuerzaLorentz[1.6e-19, 1000000, 1]",
        "1.600e-13",
    );
    assert_message_contains(
        &mut document,
        "FuerzaLorentz[1.6e-19, 1000000, 1]",
        "mano derecha",
    );
    // Electrón: mano izquierda.
    assert_message_contains(
        &mut document,
        "FuerzaLorentz[-1.6e-19, 1000000, 1]",
        "mano izquierda",
    );
    // Neutro: nula.
    assert_message_contains(&mut document, "FuerzaLorentz[0, 1000000, 1]", "nula");
}

#[test]
fn movimiento_acelerado_mruv_vivo() {
    let mut document = Document::new();
    // x₀=0, v₀=10, a=2: MRUV dibujado.
    assert_message_contains(&mut document, "MovimientoAcelerado[0, 10, 2]", "MRUV");
    // a=0: MRU sin comando aparte.
    assert_message_contains(&mut document, "MovimientoAcelerado[5, 3, 0]", "MRU");
    // Dos Functions vivas.
    let vivas = document
        .objects_iter()
        .filter(|(_, o)| matches!(o, GeoObject::Function(_)))
        .count();
    assert_eq!(vivas, 2, "una Function por cada corrida");
}

#[test]
fn orbita_circular_iss() {
    let mut document = Document::new();
    // ISS: M=5.972e24, r=6.771e6 → v≈7672 m/s, T≈5545 s (92 min).
    assert_message_contains(&mut document, "OrbitaCircular[5.972e24, 6.771e6]", "v=7672");
    assert_message_contains(&mut document, "OrbitaCircular[5.972e24, 6.771e6]", "T=5544");
    match run(&mut document, "OrbitaCircular[5.972e24, 0]") {
        CommandOutcome::Error(m) => assert!(m.contains("OrbitaCircular"), "{m}"),
        other => panic!("r=0 debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "OrbitaCircular[-1, 6.771e6]") {
        CommandOutcome::Error(m) => assert!(m.contains("OrbitaCircular"), "{m}"),
        other => panic!("M<0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn paleta_fisica_armonica_semantica() {
    use grafito_geometry::Color;
    // docs/paleta-fisica.md: cada etiqueta tiene SU color. Cambiar un
    // color rompe este test a propósito: la armonía se discute.
    let mut document = Document::new();
    run(&mut document, "PlanoInclinado[2, 30]");
    run(&mut document, "VectoresProyectil[10, 45, 0, 0.5]");
    let mut por_etiqueta = std::collections::HashMap::new();
    for (_, o) in document.objects_iter() {
        if let GeoObject::Line(l) = o {
            por_etiqueta.insert(l.label.clone(), l.color);
        }
    }
    let rojo = Color::new(0.9, 0.2, 0.2, 1.0);
    let verde = Color::new(0.2, 0.7, 0.2, 1.0);
    let naranja = Color::new(0.9, 0.6, 0.1, 1.0);
    let azul = Color::new(0.2, 0.4, 0.9, 1.0);
    let gris = Color::new(0.55, 0.55, 0.55, 1.0);
    assert_eq!(por_etiqueta.get("P"), Some(&rojo), "P es roja");
    assert_eq!(por_etiqueta.get("N"), Some(&azul), "N es azul");
    assert_eq!(por_etiqueta.get("Ppar"), Some(&naranja), "P∥ es naranja");
    assert_eq!(por_etiqueta.get("cuna_rampa"), Some(&gris), "soporte gris");
    assert_eq!(por_etiqueta.get("v"), Some(&verde), "v es verde");
    assert_eq!(por_etiqueta.get("a"), Some(&rojo), "a es roja");
    assert_eq!(por_etiqueta.get("at"), Some(&naranja), "at es naranja");
    assert_eq!(por_etiqueta.get("an"), Some(&azul), "an es azul");
}

#[test]
fn nivel_sonoro_logaritmo_no_regla_de_tres() {
    let mut document = Document::new();
    // 1e-6 W/m² = 60 dB: conversación.
    assert_message_contains(&mut document, "NivelSonoro[0.000001]", "60.0 dB");
    assert_message_contains(&mut document, "NivelSonoro[0.000001]", "conversación");
    // Umbral auditivo: 0 dB.
    assert_message_contains(&mut document, "NivelSonoro[1e-12]", "0.0 dB");
    // Silencio honesto, no −∞.
    assert_message_contains(&mut document, "NivelSonoro[0]", "silencio");
    match run(&mut document, "NivelSonoro[-1]") {
        CommandOutcome::Error(m) => assert!(m.contains("NivelSonoro"), "{m}"),
        other => panic!("I<0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn ph_acido_neutro_base() {
    let mut document = Document::new();
    // Jugo gástrico ≈ pH 2.
    assert_message_contains(&mut document, "PH[0.01]", "2.00");
    assert_message_contains(&mut document, "PH[0.01]", "ÁCIDO");
    // Neutro exacto.
    assert_message_contains(&mut document, "PH[1e-7]", "NEUTRO");
    // Base.
    assert_message_contains(&mut document, "PH[1e-10]", "10.00");
    assert_message_contains(&mut document, "PH[1e-10]", "BASE");
    // Agua pura honesta, no −∞.
    assert_message_contains(&mut document, "PH[0]", "agua pura");
    match run(&mut document, "PH[-1]") {
        CommandOutcome::Error(m) => assert!(m.contains("PH"), "{m}"),
        other => panic!("c<0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn densidad_decide_antes_que_arquimedes() {
    let mut document = Document::new();
    // Hielo: 917 kg/m³ flota.
    assert_message_contains(&mut document, "Densidad[917, 1]", "917.0");
    assert_message_contains(&mut document, "Densidad[917, 1]", "FLOTA en agua");
    // Hierro: se hunde.
    assert_message_contains(&mut document, "Densidad[7800, 1]", "SE HUNDE en agua");
    match run(&mut document, "Densidad[5, 0]") {
        CommandOutcome::Error(m) => assert!(m.contains("Densidad"), "{m}"),
        other => panic!("V=0 debe dar Error, dio {other:?}"),
    }
    match run(&mut document, "Densidad[-1, 2]") {
        CommandOutcome::Error(m) => assert!(m.contains("Densidad"), "{m}"),
        other => panic!("m<0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn trabajo_alimenta_energia() {
    let mut document = Document::new();
    // 10 N por 5 m = 50 J.
    assert_message_contains(&mut document, "Trabajo[10, 5]", "W=50.000 J");
    // Trabajo negativo honesto (frena, no falla).
    assert_message_contains(&mut document, "Trabajo[10, -5]", "W=-50.000 J");
}

#[test]
fn potencia_cierra_triptico_energia() {
    let mut document = Document::new();
    // 100 J en 4 s = 25 W.
    assert_message_contains(&mut document, "PotenciaMecanica[100, 4]", "P=25.000 W");
    match run(&mut document, "PotenciaMecanica[100, 0]") {
        CommandOutcome::Error(m) => assert!(m.contains("PotenciaMecanica"), "{m}"),
        other => panic!("t=0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn tercera_kepler_constante_tierra() {
    let mut document = Document::new();
    // k=4π²/GM con M Tierra ≈ 9.90e-14 s²/m³.
    assert_message_contains(&mut document, "TerceraKepler[5.972e24]", "Kepler III");
    assert_message_contains(&mut document, "TerceraKepler[5.972e24]", "9.90");
    assert_message_contains(&mut document, "TerceraKepler[5.972e24]", "s²/m³");
    match run(&mut document, "TerceraKepler[0]") {
        CommandOutcome::Error(m) => assert!(m.contains("TerceraKepler"), "{m}"),
        other => panic!("M=0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn fuerza_rozamiento_cierra_plano_ideal() {
    let mut document = Document::new();
    // μ=0.3, N=16.97 (el N de PlanoInclinado[2, 30]) → 5.091 N.
    assert_message_contains(&mut document, "FuerzaRozamiento[0.3, 16.97]", "fr=5.091 N");
    // Sin roce honesto, no error.
    assert_message_contains(&mut document, "FuerzaRozamiento[0, 10]", "fr=0.000 N");
    match run(&mut document, "FuerzaRozamiento[-0.1, 10]") {
        CommandOutcome::Error(m) => assert!(m.contains("FuerzaRozamiento"), "{m}"),
        other => panic!("μ<0 debe dar Error, dio {other:?}"),
    }
}

#[test]
fn campo_cargas_3d_dipolo_real_en_volumen() {
    let mut document = Document::new();
    // Dipolo en x: +1 en (-1,0,0), −1 en (1,0,0).
    // E(0,0,0) = 1·(0+1)/1 + (−1)·(0−1)/1 = 2 en x, 0 en y/z.
    assert_message_contains(
        &mut document,
        "CampoCargas3D[1, -1, 0, 0, -1, 1, 0, 0]",
        "2 carga(s)",
    );
    let vf = document
        .objects_iter()
        .find_map(|(_, o)| match o {
            GeoObject::VectorField3D(v) => Some(v),
            _ => None,
        })
        .expect("CampoCargas3D debe crear VectorField3D");
    assert!(vf.color_by_magnitude, "campo E 3D con color=|E|");
    let vars = [
        ("x".to_string(), 0.0),
        ("y".to_string(), 0.0),
        ("z".to_string(), 0.0),
    ];
    let ex = grafito_geometry::expr::evaluate(&vf.expr_u, &vars).expect("Ex evalúa");
    let ey = grafito_geometry::expr::evaluate(&vf.expr_v, &vars).expect("Ey evalúa");
    let ez = grafito_geometry::expr::evaluate(&vf.expr_w, &vars).expect("Ez evalúa");
    assert!((ex - 2.0).abs() < 1e-9, "Ex(0,0,0)={ex}");
    assert!(ey.abs() < 1e-9, "Ey(0,0,0)={ey}");
    assert!(ez.abs() < 1e-9, "Ez(0,0,0)={ez}");
    let esferas = document
        .objects_iter()
        .filter(|(_, o)| matches!(o, GeoObject::Sphere3D(_)))
        .count();
    assert_eq!(esferas, 2, "una esfera por carga");
}

#[test]
fn sistema_orbital_iss_inclinada_dibuja_plano() {
    // ISS: M Tierra, r = 6371+400 km, inc 51.6° (misma cuenta que OrbitaCircular).
    let mut document = Document::new();
    assert_message_contains(
        &mut document,
        "SistemaOrbital[5.972e24, 6.771e6, 51.6]",
        "v=7672.490 m/s",
    );
    let mut document = Document::new();
    assert_message_contains(
        &mut document,
        "SistemaOrbital[5.972e24, 6.771e6, 51.6]",
        "T=5544",
    );
    // 3 objetos: estrella + órbita + satélite, paleta cerrada.
    let esferas = document
        .objects_iter()
        .filter(|(_, o)| matches!(o, GeoObject::Sphere3D(_)))
        .count();
    assert_eq!(esferas, 2, "estrella + satélite");
    let orbita = document
        .objects_iter()
        .find_map(|(_, o)| match o {
            GeoObject::ParametricCurve3D(c) => Some(c),
            _ => None,
        })
        .expect("SistemaOrbital debe crear la órbita 3D");
    // En t=π/2 la inclinación queda horneada: y=5·cos(51.6°), z=5·sin(51.6°).
    let vars = [("t".to_string(), std::f64::consts::FRAC_PI_2)];
    let y = grafito_geometry::expr::evaluate(&orbita.expr_y, &vars).expect("Ey evalúa");
    let z = grafito_geometry::expr::evaluate(&orbita.expr_z, &vars).expect("Ez evalúa");
    assert!((y - 3.107).abs() < 0.01, "y(π/2)={y}");
    assert!((z - 3.919).abs() < 0.01, "z(π/2)={z}");
    // Errores honestos con el nombre del comando.
    for mala in [
        "SistemaOrbital[0, 6.771e6, 51.6]",
        "SistemaOrbital[5.972e24, 6.771e6, 100]",
        "SistemaOrbital[5.972e24, 6.771e6, -5]",
    ] {
        match run(&mut document, mala) {
            CommandOutcome::Error(m) => assert!(m.contains("SistemaOrbital"), "{m}"),
            other => panic!("{mala} debe dar Error, dio {other:?}"),
        }
    }
}

#[test]
fn velocidad_onda_dos_fotos_muestran_avance() {
    // λ=2, f=3 → v=6, T=1/3. Dos fotos: t=0 verde, t=T/4 gris.
    let mut document = Document::new();
    assert_message_contains(&mut document, "VelocidadOnda[2, 3]", "v=6.000 m/s");
    assert_message_contains(&mut document, "VelocidadOnda[2, 3]", "T=0.333 s");
    let mut document = Document::new();
    assert_message_contains(&mut document, "VelocidadOnda[2, 3]", "gris = T/4");
    let funciones: Vec<_> = document
        .objects_iter()
        .filter_map(|(_, o)| match o {
            GeoObject::Function(f) => Some(f),
            _ => None,
        })
        .collect();
    assert_eq!(funciones.len(), 2, "t=0 + t=T/4");
    // Sin orden asumido (objects_iter no garantiza inserción): se
    // identifican por la expresión, no por el índice.
    let ahora = funciones
        .iter()
        .find(|f| !f.expr.contains("-pi/2"))
        .expect("foto t=0");
    let despues = funciones
        .iter()
        .find(|f| f.expr.contains("-pi/2"))
        .expect("foto t=T/4 con desfase");
    assert_ne!(ahora.color, despues.color, "verde vs gris distinguibles");
    // En x=λ/4 la primera está en el pico: sin(2π·0.5/2)=sin(π/2)=1.
    let vars = [("x".to_string(), 0.5)];
    let y = grafito_geometry::expr::evaluate(&ahora.expr, &vars).expect("onda evalúa");
    assert!((y - 1.0).abs() < 1e-9, "y(λ/4)={y}");
    for mala in [
        "VelocidadOnda[0, 3]",
        "VelocidadOnda[2, -1]",
        "VelocidadOnda[2, 0]",
    ] {
        match run(&mut document, mala) {
            CommandOutcome::Error(m) => assert!(m.contains("VelocidadOnda"), "{m}"),
            other => panic!("{mala} debe dar Error, dio {other:?}"),
        }
    }
}

#[test]
fn cantidad_movimiento_p_igual_m_por_v() {
    // Auto de 1000 kg a 20 m/s → 20000 kg·m/s (lo que el choque conserva).
    let mut document = Document::new();
    assert_message_contains(
        &mut document,
        "CantidadMovimiento[1000, 20]",
        "p=20000.000 kg·m/s",
    );
    // Reposo honesto, no error.
    assert_message_contains(
        &mut document,
        "CantidadMovimiento[1000, 0]",
        "p=0.000 kg·m/s",
    );
    // Solo lectura: no crea objetos.
    assert_eq!(document.objects_iter().count(), 0, "sin dibujo");
    for mala in ["CantidadMovimiento[-1, 20]"] {
        match run(&mut document, mala) {
            CommandOutcome::Error(m) => assert!(m.contains("CantidadMovimiento"), "{m}"),
            other => panic!("{mala} debe dar Error, dio {other:?}"),
        }
    }
}

#[test]
fn aceleracion_centripeta_v_cuadrado_sobre_r() {
    // Auto a 20 m/s en curva de 50 m → 8 m/s² hacia el centro.
    let mut document = Document::new();
    assert_message_contains(
        &mut document,
        "AceleracionCentripeta[20, 50]",
        "a=8.000 m/s²",
    );
    // Solo lectura: no crea objetos.
    assert_eq!(document.objects_iter().count(), 0, "sin dibujo");
    for mala in [
        "AceleracionCentripeta[20, 0]",
        "AceleracionCentripeta[20, -5]",
    ] {
        match run(&mut document, mala) {
            CommandOutcome::Error(m) => assert!(m.contains("AceleracionCentripeta"), "{m}"),
            other => panic!("{mala} debe dar Error, dio {other:?}"),
        }
    }
}

#[test]
fn ley_gravitacion_tierra_luna_con_g_real() {
    // Tierra–Luna: G·m1·m2/r² ≈ 1.980e20 N (el número detrás de la órbita).
    let mut document = Document::new();
    assert_message_contains(
        &mut document,
        "LeyGravitacion[5.972e24, 7.342e22, 3.844e8]",
        "F=1.980e20 N",
    );
    // Solo lectura: no crea objetos.
    assert_eq!(document.objects_iter().count(), 0, "sin dibujo");
    for mala in [
        "LeyGravitacion[5.972e24, 7.342e22, 0]",
        "LeyGravitacion[-1, 7.342e22, 3.844e8]",
    ] {
        match run(&mut document, mala) {
            CommandOutcome::Error(m) => assert!(m.contains("LeyGravitacion"), "{m}"),
            other => panic!("{mala} debe dar Error, dio {other:?}"),
        }
    }
}

#[test]
fn energia_bohr_niveles_del_hidrogeno() {
    // n=1 ata (−13.6 eV), n=2 ya afloja (−3.4 eV).
    let mut document = Document::new();
    assert_message_contains(&mut document, "EnergiaBohr[1]", "E=-13.600 eV");
    assert_message_contains(&mut document, "EnergiaBohr[2]", "E=-3.400 eV");
    // Solo lectura: no crea objetos.
    assert_eq!(document.objects_iter().count(), 0, "sin dibujo");
    for mala in ["EnergiaBohr[0]", "EnergiaBohr[-1]", "EnergiaBohr[1.5]"] {
        match run(&mut document, mala) {
            CommandOutcome::Error(m) => assert!(m.contains("EnergiaBohr"), "{m}"),
            other => panic!("{mala} debe dar Error, dio {other:?}"),
        }
    }
}

#[test]
fn ley_faraday_fem_contra_el_cambio() {
    // N=100, ΔΦ=0.01 Wb en 0.5 s: fem=−2 V (Lenz: opone la suba).
    let mut document = Document::new();
    assert_message_contains(&mut document, "LeyFaraday[100, 0.01, 0.5]", "fem=-2.000 V");
    // Solo lectura: no crea objetos.
    assert_eq!(document.objects_iter().count(), 0, "sin dibujo");
    for mala in [
        "LeyFaraday[0, 0.01, 0.5]",
        "LeyFaraday[1.5, 0.01, 0.5]",
        "LeyFaraday[100, 0.01, 0]",
        "LeyFaraday[100, 0.01, -1]",
    ] {
        match run(&mut document, mala) {
            CommandOutcome::Error(m) => assert!(m.contains("LeyFaraday"), "{m}"),
            other => panic!("{mala} debe dar Error, dio {other:?}"),
        }
    }
}

#[test]
fn flujo_gauss_carga_manda_el_flujo() {
    // Q=1 nC: Φ=1e-9/ε₀=112.941 N·m²/C (el flujo sale de la carga).
    let mut document = Document::new();
    assert_message_contains(&mut document, "FlujoGauss[1e-9]", "Φ=112.941 N·m²/C");
    // Q=0 válido (caja vacía, flujo cero); solo lectura, sin dibujo.
    assert_message_contains(&mut document, "FlujoGauss[0]", "Φ=0.000 N·m²/C");
    assert_eq!(document.objects_iter().count(), 0, "sin dibujo");
    // Alias con guion bajo + error honesto con no-finito.
    assert_message_contains(&mut document, "flujo_gauss[1e-9]", "Φ=112.941 N·m²/C");
    match run(&mut document, "FlujoGauss[inf]") {
        CommandOutcome::Error(m) => assert!(m.contains("FlujoGauss"), "{m}"),
        other => panic!("FlujoGauss[inf] debe dar Error, dio {other:?}"),
    }
}

#[test]
fn calor_latente_fundir_sin_calentar() {
    // 1 kg de hielo (L=334000 J/kg): Q=334000 J sin subir un grado.
    let mut document = Document::new();
    assert_message_contains(&mut document, "CalorLatente[1, 334000]", "Q=334000.000 J");
    // Solo lectura: no crea objetos.
    assert_eq!(document.objects_iter().count(), 0, "sin dibujo");
    // Alias con guion bajo.
    assert_message_contains(&mut document, "calor_latente[2, 334000]", "Q=668000.000 J");
    for mala in [
        "CalorLatente[0, 334000]",
        "CalorLatente[-1, 334000]",
        "CalorLatente[1, 0]",
        "CalorLatente[1, -5]",
    ] {
        match run(&mut document, mala) {
            CommandOutcome::Error(m) => assert!(m.contains("CalorLatente"), "{m}"),
            other => panic!("{mala} debe dar Error, dio {other:?}"),
        }
    }
}

#[test]
fn choque_elastico_intercambian_velocidades() {
    // Masas iguales: intercambian velocidades (el de atrás frena en seco).
    let mut document = Document::new();
    assert_message_contains(
        &mut document,
        "ChoqueElastico[1, 3, 1, 0]",
        "v1′=0.000 v2′=3.000 m/s",
    );
    // Solo lectura: no crea objetos.
    assert_eq!(document.objects_iter().count(), 0, "sin dibujo");
    // Pesado contra liviano: el liviano sale casi al doble.
    assert_message_contains(
        &mut document,
        "ChoqueElastico[10, 2, 1, 0]",
        "v1′=1.636 v2′=3.636 m/s",
    );
    // Alias con guion bajo.
    assert_message_contains(
        &mut document,
        "choque_elastico[1, 3, 1, 0]",
        "v1′=0.000 v2′=3.000 m/s",
    );
    for mala in [
        "ChoqueElastico[0, 3, 1, 0]",
        "ChoqueElastico[-1, 3, 1, 0]",
        "ChoqueElastico[1, 3, 0, 0]",
    ] {
        match run(&mut document, mala) {
            CommandOutcome::Error(m) => assert!(m.contains("ChoqueElastico"), "{m}"),
            other => panic!("{mala} debe dar Error, dio {other:?}"),
        }
    }
}

#[test]
fn velocidad_sonido_aire_a_veinte_grados() {
    // El número que todo libro cita: 343.215 m/s a 20 °C.
    let mut document = Document::new();
    assert_message_contains(&mut document, "VelocidadSonido[20]", "v=343.215 m/s");
    // Solo lectura: no crea objetos.
    assert_eq!(document.objects_iter().count(), 0, "sin dibujo");
    // Alias con guion bajo.
    assert_message_contains(&mut document, "velocidad_sonido[0]", "v=331.300 m/s");
    // Bajo el cero absoluto no hay aire que vibre.
    match run(&mut document, "VelocidadSonido[-300]") {
        CommandOutcome::Error(m) => assert!(m.contains("VelocidadSonido"), "{m}"),
        other => panic!("VelocidadSonido[-300] debe dar Error, dio {other:?}"),
    }
}

#[test]
fn alcance_proyectil_cuarenta_y_cinco_rinde_mas() {
    // 20 m/s a 45°: R=40.775 m (sin rozamiento).
    let mut document = Document::new();
    assert_message_contains(&mut document, "AlcanceProyectil[20, 45]", "R=40.775 m");
    // Solo lectura: no crea objetos.
    assert_eq!(document.objects_iter().count(), 0, "sin dibujo");
    // A 30° cae menos: simetría del sin(2θ).
    assert_message_contains(&mut document, "AlcanceProyectil[20, 30]", "R=35.312 m");
    // Alias con guion bajo.
    assert_message_contains(&mut document, "alcance_proyectil[20, 45]", "R=40.775 m");
    match run(&mut document, "AlcanceProyectil[-5, 45]") {
        CommandOutcome::Error(m) => assert!(m.contains("AlcanceProyectil"), "{m}"),
        other => panic!("AlcanceProyectil[-5, 45] debe dar Error, dio {other:?}"),
    }
}

#[test]
fn energia_capacitor_guarda_en_silencio() {
    // 1 mF a 100 V: E=5 J (el flash de la cámara vieja).
    let mut document = Document::new();
    assert_message_contains(&mut document, "EnergiaCapacitor[0.001, 100]", "E=5.000 J");
    // Solo lectura: no crea objetos.
    assert_eq!(document.objects_iter().count(), 0, "sin dibujo");
    // El signo de U no importa: va al cuadrado.
    assert_message_contains(&mut document, "EnergiaCapacitor[0.001, -100]", "E=5.000 J");
    // Alias con guion bajo.
    assert_message_contains(&mut document, "energia_capacitor[0.002, 100]", "E=10.000 J");
    for mala in ["EnergiaCapacitor[0, 100]", "EnergiaCapacitor[-1, 100]"] {
        match run(&mut document, mala) {
            CommandOutcome::Error(m) => assert!(m.contains("EnergiaCapacitor"), "{m}"),
            other => panic!("{mala} debe dar Error, dio {other:?}"),
        }
    }
}

//! Export DIMACS para verificación externa TOPP 57 (Fase B.3).
//!
//! Uso: `cargo run -p grafito-geometry --example export_dimacs -- [dir]`
//! Default: `/tmp/opencode/b3`. Escribe dos CNF con el exportador propio
//! (`export_dimacs_kcoloring`) y NADA más: este ejemplo no declara
//! SAT/UNSAT (la palabra final la tiene kissat/cadical vía `sat_check`).
//! Instancias testigo (respuestas conocidas solo por la literatura):
//! K4 con k=3 y triángulo con k=3.
//!
//! Sin red, sin UI: solo el motor puro + dos archivos.

use grafito_geometry::search::export_dimacs_kcoloring;

fn k4_edges() -> Vec<(usize, usize)> {
    let mut edges = Vec::new();
    for a in 0..4 {
        for b in (a + 1)..4 {
            edges.push((a, b));
        }
    }
    edges
}

fn write_cnf(dir: &std::path::Path, name: &str, cnf: &str) -> std::io::Result<std::path::PathBuf> {
    let path = dir.join(name);
    std::fs::write(&path, cnf)?;
    Ok(path)
}

fn header_summary(cnf: &str) -> String {
    cnf.lines().next().unwrap_or("p cnf ? ?").trim().to_string()
}

fn run(dir: &std::path::Path) -> Result<(), String> {
    struct Caso {
        nombre: &'static str,
        n: usize,
        edges: Vec<(usize, usize)>,
        k: usize,
    }
    std::fs::create_dir_all(dir).map_err(|e| format!("mkdir {}: {e}", dir.display()))?;
    let casos: Vec<Caso> = vec![
        Caso {
            nombre: "k4_k3.cnf",
            n: 4,
            edges: k4_edges(),
            k: 3,
        },
        Caso {
            nombre: "tri_k3.cnf",
            n: 3,
            edges: vec![(0, 1), (1, 2), (0, 2)],
            k: 3,
        },
    ];
    for caso in &casos {
        let cnf = export_dimacs_kcoloring(caso.n, &caso.edges, caso.k)
            .map_err(|e| format!("export {}: {e}", caso.nombre))?;
        let path =
            write_cnf(dir, caso.nombre, &cnf).map_err(|e| format!("write {}: {e}", caso.nombre))?;
        println!(
            "{} n={} m={} k={} {}",
            path.display(),
            caso.n,
            caso.edges.len(),
            caso.k,
            header_summary(&cnf)
        );
    }
    Ok(())
}

fn main() {
    let dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/tmp/opencode/b3".to_string());
    if let Err(e) = run(std::path::Path::new(&dir)) {
        eprintln!("export_dimacs: {e}");
        std::process::exit(1);
    }
}

//! Plan de examen E3 (producto Estudio): dato puro en el perfil.
//!
//! `StudyPlan` guarda materia, fecha, hs/semana, dominio BKT por tema
//! (el `f64` lo calcula la app con `bkt_update`; acá solo se promedia) y
//! snapshots de readiness para proyectar. Sin reloj dentro: el `today`
//! (días desde epoch) lo pasa la app — tests deterministas.
//! Progreso honesto: `readiness()` = promedio ponderado BKT; sin
//! respuestas es `None` ("sin datos", nunca 0%).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Umbral de "llegás": probabilidad 70% de aprobar.
pub const READY_THRESHOLD: f32 = 0.7;
/// Valor inicial de dominio BKT para un tema nuevo.
pub const INITIAL_MASTERY: f64 = 0.3;

/// Un día del plan: minutos sugeridos + temas más flojos primero.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanDay {
    /// Días desde epoch (absoluto, comparable con `today`).
    pub day: u64,
    pub minutes: u32,
    pub topics: Vec<String>,
}

/// Veredicto de proyección E3/E5 (fail-closed docente).
#[derive(Debug, Clone, PartialEq)]
pub enum Projection {
    /// Sin respuestas: el modelo no sabe, lo dice.
    SinDatos,
    /// Hay dominio pero sin tasa (menos de 2 snapshots).
    EnCamino { readiness: f32, days_left: u64 },
    /// La tasa lineal alcanza el umbral antes del examen.
    Llegas { readiness: f32, days_left: u64 },
    /// La tasa lineal no alcanza: fecha proyectada del 70%.
    NoLlegas {
        readiness: f32,
        projected_day: u64,
        exam_day: u64,
    },
}

/// Plan de examen persistido en `StudentProfile.study_plan`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StudyPlan {
    pub subject: String,
    /// Día del examen (días desde epoch UTC).
    pub exam_day: u64,
    pub hours_per_week: f32,
    /// Temas del plan (los del banco + los que agregue el quiz).
    pub topics: Vec<String>,
    /// Dominio BKT por tema (`p_known` 0..1).
    pub mastery: HashMap<String, f64>,
    /// Respuestas acumuladas (denominador de confianza).
    pub answers: u64,
    /// `(día, readiness)` tras cada quiz registrado (tasa de proyección).
    pub snapshots: Vec<(u64, f32)>,
}

impl StudyPlan {
    pub fn new(subject: &str, exam_day: u64, hours_per_week: f32) -> Self {
        Self {
            subject: subject.to_string(),
            exam_day,
            hours_per_week: hours_per_week.max(0.5),
            topics: Vec::new(),
            mastery: HashMap::new(),
            answers: 0,
            snapshots: Vec::new(),
        }
    }

    /// Días restantes (0 si ya pasó; nunca negativo).
    #[must_use]
    pub fn days_left(&self, today: u64) -> u64 {
        self.exam_day.saturating_sub(today)
    }

    /// Registra una respuesta: crea el tema si no existe y acumula.
    pub fn record_answer(&mut self, topic: &str, p_known: f64) {
        if !self.topics.iter().any(|t| t == topic) {
            self.topics.push(topic.to_string());
        }
        self.mastery
            .insert(topic.to_string(), p_known.clamp(0.0, 1.0));
        self.answers += 1;
    }

    /// Foto del readiness actual (la app la llama tras cada quiz).
    pub fn snapshot(&mut self, today: u64) {
        if let Some(r) = self.readiness() {
            self.snapshots.push((today, r));
        }
    }

    /// Readiness = promedio simple del dominio por tema. `None` sin
    /// respuestas: "sin datos", nunca un 0% mentiroso.
    #[must_use]
    pub fn readiness(&self) -> Option<f32> {
        if self.answers == 0 || self.topics.is_empty() {
            return None;
        }
        let sum: f64 = self
            .topics
            .iter()
            .map(|t| self.mastery.get(t).copied().unwrap_or(INITIAL_MASTERY))
            .sum();
        Some((sum / self.topics.len() as f64) as f32)
    }

    /// Proyección lineal sobre snapshots (≥2): tasa diaria de ganancia.
    #[must_use]
    pub fn project(&self, today: u64) -> Projection {
        let days_left = self.days_left(today);
        let Some(readiness) = self.readiness() else {
            return Projection::SinDatos;
        };
        if readiness >= READY_THRESHOLD {
            return Projection::Llegas {
                readiness,
                days_left,
            };
        }
        if self.snapshots.len() < 2 {
            return Projection::EnCamino {
                readiness,
                days_left,
            };
        }
        let (d0, r0) = self.snapshots[0];
        let (d1, r1) = self.snapshots[self.snapshots.len() - 1];
        let span = d1.saturating_sub(d0).max(1) as f32;
        let rate = (r1 - r0) / span;
        if rate <= 0.0 {
            return Projection::EnCamino {
                readiness,
                days_left,
            };
        }
        let need = READY_THRESHOLD - readiness;
        let projected_day = today + (need / rate).ceil().max(0.0) as u64;
        if projected_day <= self.exam_day {
            Projection::Llegas {
                readiness,
                days_left,
            }
        } else {
            Projection::NoLlegas {
                readiness,
                projected_day,
                exam_day: self.exam_day,
            }
        }
    }

    /// Reparte las horas en los días restantes, rotando los temas más
    /// flojos primero (2 por día). Vacío si no quedan días.
    #[must_use]
    pub fn plan_days(&self, today: u64) -> Vec<PlanDay> {
        let days = self.days_left(today);
        if days == 0 {
            return Vec::new();
        }
        let per_day = (self.hours_per_week * 60.0 / 7.0).round().max(10.0) as u32;
        let mut weak: Vec<&String> = self.topics.iter().collect();
        weak.sort_by(|a, b| {
            self.mastery
                .get(*a)
                .copied()
                .unwrap_or(INITIAL_MASTERY)
                .partial_cmp(&self.mastery.get(*b).copied().unwrap_or(INITIAL_MASTERY))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        (0..days)
            .map(|o| {
                let pick: Vec<String> = weak.iter().take(2).map(|s| (*s).clone()).collect();
                // Rota para que todos los temas circulen por el plan.
                if weak.len() > 1 {
                    weak.rotate_left(1);
                }
                PlanDay {
                    day: today + o,
                    minutes: per_day,
                    topics: pick,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sin_respuestas_sin_datos() {
        let plan = StudyPlan::new("Física", 100, 6.0);
        assert_eq!(plan.readiness(), None);
        assert_eq!(plan.project(90), Projection::SinDatos);
        assert!(plan.plan_days(90).len() == 10);
    }

    #[test]
    fn readiness_promedia_temas() {
        let mut plan = StudyPlan::new("Física", 100, 6.0);
        plan.record_answer("Ondas", 0.8);
        plan.record_answer("Tiro", 0.4);
        let r = plan.readiness().expect("hay respuestas");
        assert!((r - 0.6).abs() < 1e-6, "readiness={r}");
    }

    #[test]
    fn llegas_con_umbral() {
        let mut plan = StudyPlan::new("Física", 100, 6.0);
        plan.record_answer("Ondas", 0.9);
        plan.record_answer("Tiro", 0.8);
        assert!(matches!(plan.project(90), Projection::Llegas { .. }));
    }

    #[test]
    fn proyeccion_lineal_no_llega() {
        let mut plan = StudyPlan::new("Física", 110, 2.0);
        plan.record_answer("Ondas", 0.4);
        plan.snapshot(100);
        plan.record_answer("Ondas", 0.41);
        plan.snapshot(101);
        match plan.project(101) {
            Projection::NoLlegas { projected_day, .. } => {
                assert!(projected_day > 110, "proyecta más allá del examen");
            }
            other => panic!("esperaba NoLlegas, dio {other:?}"),
        }
    }

    #[test]
    fn dias_pasados_plan_vacio() {
        let plan = StudyPlan::new("Física", 100, 6.0);
        assert_eq!(plan.days_left(101), 0);
        assert!(plan.plan_days(101).is_empty());
    }
}

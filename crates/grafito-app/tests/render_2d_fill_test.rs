#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(deprecated)]
//! Verifica que el render de un ImplicitCurve con Eq dibuja el outline.

#[cfg(test)]
mod tests {
    use grafito_app::render_2d::implicit_curve_requests_fill;
    use grafito_core::{ImplicitCurveObj, RelationOperator};

    fn curve_with(op: RelationOperator) -> ImplicitCurveObj {
        ImplicitCurveObj::new("x^2 + y^2", "1", op)
    }

    #[test]
    fn test_eq_does_not_call_fill() {
        // Llama a la decisión real de producción: si `render_2d` invirtiera
        // la condición, este test falla (antes duplicaba el `matches!` y
        // pasaba siempre).
        assert!(
            !implicit_curve_requests_fill(&curve_with(RelationOperator::Eq)),
            "Eq no debe activar el fill"
        );
    }

    #[test]
    fn test_inequalities_call_fill() {
        for op in [
            RelationOperator::Less,
            RelationOperator::LessEq,
            RelationOperator::Greater,
            RelationOperator::GreaterEq,
        ] {
            assert!(
                implicit_curve_requests_fill(&curve_with(op)),
                "{op:?} debe activar el fill"
            );
        }
    }
}

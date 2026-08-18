use rust_decimal::Decimal;

/// Coût de revient par unité produite : total des coûts du mois / quantité produite.
/// `None` si aucune production n'a été déclarée pour ce mois (division impossible).
pub fn cost_per_unit(total_cost: Decimal, quantity_produced: Decimal) -> Option<Decimal> {
    if quantity_produced.is_zero() {
        None
    } else {
        Some(total_cost / quantity_produced)
    }
}

/// Marge estimée = (quantité produite * prix de vente unitaire) - total des coûts.
/// `None` si aucun prix de vente n'a été renseigné pour ce mois.
pub fn estimated_margin(
    total_cost: Decimal,
    quantity_produced: Decimal,
    unit_sale_price: Option<Decimal>,
) -> Option<Decimal> {
    unit_sale_price.map(|price| (quantity_produced * price) - total_cost)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn cost_per_unit_divides_total_cost_by_quantity() {
        assert_eq!(cost_per_unit(dec!(100), dec!(4)), Some(dec!(25)));
    }

    #[test]
    fn cost_per_unit_is_none_when_no_production() {
        assert_eq!(cost_per_unit(dec!(100), dec!(0)), None);
    }

    #[test]
    fn estimated_margin_is_none_without_a_sale_price() {
        assert_eq!(estimated_margin(dec!(100), dec!(4), None), None);
    }

    #[test]
    fn estimated_margin_subtracts_cost_from_revenue() {
        assert_eq!(
            estimated_margin(dec!(100), dec!(4), Some(dec!(30))),
            Some(dec!(20))
        );
    }
}

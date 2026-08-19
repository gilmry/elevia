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

/// Marge estimée = (quantité **vendue** * prix de vente unitaire) - total des coûts.
///
/// Volontairement basée sur ce qui a été vendu, pas sur ce qui a été produit :
/// tout ce qui est produit dans le mois n'est pas forcément vendu ce même
/// mois (stock, vente différée, perte...), donc utiliser la quantité produite
/// surestimerait systématiquement la marge dès qu'il reste du stock.
/// `None` si la quantité vendue ou le prix de vente n'est pas renseigné.
pub fn estimated_margin(
    total_cost: Decimal,
    quantity_sold: Option<Decimal>,
    unit_sale_price: Option<Decimal>,
) -> Option<Decimal> {
    let quantity_sold = quantity_sold?;
    let price = unit_sale_price?;
    Some((quantity_sold * price) - total_cost)
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
        assert_eq!(estimated_margin(dec!(100), Some(dec!(4)), None), None);
    }

    #[test]
    fn estimated_margin_is_none_without_a_quantity_sold() {
        assert_eq!(estimated_margin(dec!(100), None, Some(dec!(30))), None);
    }

    #[test]
    fn estimated_margin_subtracts_cost_from_revenue() {
        assert_eq!(
            estimated_margin(dec!(100), Some(dec!(4)), Some(dec!(30))),
            Some(dec!(20))
        );
    }

    #[test]
    fn estimated_margin_uses_quantity_sold_not_quantity_produced() {
        // Produced 10 but only sold 4 - margin must reflect the 4 actually sold,
        // not the 10 produced (the rest is stock, not revenue yet).
        assert_eq!(
            estimated_margin(dec!(100), Some(dec!(4)), Some(dec!(30))),
            Some(dec!(20))
        );
    }
}

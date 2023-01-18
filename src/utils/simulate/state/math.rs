use ethers::prelude::*;
use uniswap_v3_math::tick_math::*;

// Return a human readable price from sqrt_ratio_x96.
/*
 * @dev sqrt_ratio_x96 = token_0_price.pow(-2) * 2.pow(96)
 * @dev token_0_price = 0_to_1_amount * token1_decimal / token0_decimal
 *
 * @return U256 amount (0_to_1_amount or 1_to_0_amount)
 */
pub fn sqrt_ratio_x96_price(
    (token0_decimal, token1_decimal): (u8, u8),
    sqrt_ratio_x96: U256,
) -> Result<U256, ()> {
    let a = U512::from(sqrt_ratio_x96).pow(U512::from(2))
        * U512::from(10).pow(U512::from(token0_decimal));
    let b = U512::from(2).pow(U512::from(192)) * U512::from(10).pow(U512::from(token1_decimal));
    let price = if a > b { a / b } else { b / a };

    price.try_into().or(Err(()))
}

// Return a human readable price from tick.
pub fn tick_price((token0_decimal, token1_decimal): (u8, u8), tick: i32) -> Result<U256, ()> {
    let sqrt_ratio_x96 = get_sqrt_ratio_at_tick(tick).or(Err(()))?;
    sqrt_ratio_x96_price((token0_decimal, token1_decimal), sqrt_ratio_x96)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DECIMAL_0: u8 = 18;
    const DECIMAL_1: u8 = 6;
    const SQRT_RATIO_X96_EQUAL_PRICE_ONE: u128 = 79228162514264337593543;
    const TICK_EQUAL_PRICE_ONE: i32 = -276325;

    #[test]
    fn test_sqrt_ratio_x96_price() {
        assert_eq!(
            sqrt_ratio_x96_price(
                (DECIMAL_0, DECIMAL_1),
                U256::from(SQRT_RATIO_X96_EQUAL_PRICE_ONE)
            ),
            Ok(U256::one())
        )
    }

    #[test]
    fn test_wrong_decimal_order_with_wrong_result() {
        assert!(
            sqrt_ratio_x96_price(
                (DECIMAL_1, DECIMAL_0),
                U256::from(SQRT_RATIO_X96_EQUAL_PRICE_ONE)
            )
            .unwrap()
                != U256::one()
        )
    }

    #[test]
    fn test_tick_price2() {
        assert_eq!(
            tick_price((DECIMAL_0, DECIMAL_1), TICK_EQUAL_PRICE_ONE),
            Ok(U256::one())
        );
    }

    #[test]
    fn test_wrong_decimal_order_with_wrong_result2() {
        assert!(tick_price((DECIMAL_1, DECIMAL_0), TICK_EQUAL_PRICE_ONE).unwrap() != U256::one())
    }
}

use uint::construct_uint;

// 定义 U256 以支持高精度计算 (避免 u128 溢出)
construct_uint! {
    pub struct U256(4);
}

// Constant Product AMM Calculator
// Formula: (x + dx) * (y - dy) = x * y = k
// dy = y - (x * y) / (x + dx)
// dy = (y * dx) / (x + dx)
// With Fees: dx_effective = dx * (1 - fee)

pub fn get_amount_out(
    amount_in: u64,
    reserve_in: u64,
    reserve_out: u64,
    fee_numerator: u64,
    fee_denominator: u64,
) -> Option<u64> {
    if amount_in == 0 || reserve_in == 0 || reserve_out == 0 {
        return None;
    }

    let amount_in_u256 = U256::from(amount_in);
    let reserve_in_u256 = U256::from(reserve_in);
    let reserve_out_u256 = U256::from(reserve_out);
    let fee_num = U256::from(fee_numerator);
    let fee_den = U256::from(fee_denominator);

    // 1. Calculate effective input amount (after fees)
    // amount_in_with_fee = amount_in * (denominator - numerator)
    let fee_multiplier = fee_den.checked_sub(fee_num)?;
    let amount_in_with_fee = amount_in_u256.checked_mul(fee_multiplier)?;

    // 2. Calculate numerator = amount_in_with_fee * reserve_out
    let numerator = amount_in_with_fee.checked_mul(reserve_out_u256)?;

    // 3. Calculate denominator = (reserve_in * fee_denominator) + amount_in_with_fee
    let denominator_base = reserve_in_u256.checked_mul(fee_den)?;
    let denominator = denominator_base.checked_add(amount_in_with_fee)?;

    // 4. Calculate amount_out = numerator / denominator
    let amount_out = numerator.checked_div(denominator)?;

    if amount_out > U256::from(u64::MAX) {
        None
    } else {
        Some(amount_out.as_u64())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_product_swap() {
        // Example:
        // Reserve In: 1000
        // Reserve Out: 1000
        // Amount In: 100
        // Fee: 0.25% (25/10000)
        
        // Expected without fee: 1000 - (1000*1000)/(1000+100) = 1000 - 909.09 = 90.9
        // With fee: Input effective = 100 * (1 - 0.0025) = 99.75
        // Output = (99.75 * 1000) / (1000 + 99.75) = 99750 / 1099.75 = 90.702
        
        let amount_in = 100;
        let reserve_in = 1000;
        let reserve_out = 1000;
        let fee_num = 25;
        let fee_den = 10000;
        
        let out = get_amount_out(amount_in, reserve_in, reserve_out, fee_num, fee_den).unwrap();
        assert_eq!(out, 90); // Integer math truncation
    }
}

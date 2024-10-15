//! # MIL-1750A Converter
//! Use this tool to convert to and from `MIL-1750A`
//!
//! Based on [this perl library](https://metacpan.org/release/JTCLARKE/Convert-MIL1750A-0.1/source).

use half::f16;

/// Transform 16-bit floating point number to MIL-1750A Hex
///
/// # Arguments
///
/// * `input`: number as 16-bit floating point
///
/// returns: u16 interpret as hex
///
/// # Examples
///
/// ```
/// use half::f16;
/// use MIL1750A_Converter::f16_to_1750a;
/// assert_eq!(f16_to_1750a(f16::from_f32(25.63)), 0x6685);
/// ```
pub fn f16_to_1750a(input: f16) -> u16 {
    let f32_input = f32::from(input);
    let mut exponent = f32_input.abs().log2().ceil() as i32;
    let mut mantissa = (f32_input * 2f32.powi(9 - exponent)).round() as i32;

    // Boundary check
    if mantissa == 32768 {
        mantissa /= 2;
        exponent += 1;
    }

    let mantissa_bits = ((mantissa as u16) & 0x3FF) << 6;
    let exponent_bits = (exponent as u16) & 0x3F;

    mantissa_bits | exponent_bits
}

/// Transform 32-bit floating point number to MIL-1750A Hex
///
/// # Arguments
///
/// * `input`: number as 32-bit floating point
///
/// returns: u32 interpret as hex
///
/// # Examples
///
/// ```
/// use MIL1750A_Converter::f32_to_1750a;
/// assert_eq!(f32_to_1750a(5.234), 0x53BE7703);
/// ```
pub fn f32_to_1750a(input: f32) -> u32 {
    if input == 0.0 {
        return 0;
    }

    let mut exponent = input.abs().log2().ceil() as i32;
    let mut mantissa = (input * 2f32.powi(23 - exponent)).round() as i32;

    // Boundary check
    if mantissa == 8388608 {
        mantissa /= 2;
        exponent += 1;
    }

    let mut result = (mantissa as u32) << 8;
    result |= (exponent as u32) & 0xFF;

    if input.is_sign_negative() {
        result |= 0x80000000;
    }

    result
}


/// Transform 48-bit floating point number to MIL-1750A Hex
///
/// # Arguments
///
/// * `input`: number as 48-bit (actually f64) floating point
///
/// returns: u64 interpret as hex
///
/// # Examples
///
/// ```
/// use MIL1750A_Converter::f48_to_1750a;
/// assert_eq!(f48_to_1750a(105.639485637361), 0x69A3B50754AB);
/// ```
pub fn f48_to_1750a(input: f64) -> u64 {
    if input == 0.0 {
        return 0;
    }

    let mut exponent = input.abs().log2().ceil() as i32;
    let mut mantissa = (input * 2f64.powi(39 - exponent)).round() as i64;

    // Boundary check
    if mantissa == 549755813888 {
        mantissa /= 2;
        exponent += 1;
    }

    let mantissa1 = ((mantissa >> 16) & 0xFFFFFF) as u32;
    let mantissa2 = (mantissa & 0xFFFF) as u16;
    let exponent = exponent as u8;

    let mut result = (mantissa1 as u64) << 24;
    result |= (exponent as u64) << 16;
    result |= mantissa2 as u64;

    if input.is_sign_negative() {
        result |= 0x800000000000;
    }

    result
}


/// Convert MIL-1750A hex (interpreted as u16) to f16
///
/// # Arguments
///
/// * `input`:  MIL-1750A hex (interpreted as u16)
///
/// returns: f16 representation of the input
///
/// # Examples
///
/// ```
/// use half::f16;
/// use MIL1750A_Converter::m1750a_to_16flt;
/// assert_eq!(m1750a_to_16flt(0x6344), f16::from_f32(12.40625));
/// ```
pub fn m1750a_to_16flt(input: u16) -> f16 {
    let mantissa = ((input >> 6) & 0x3FF) as f32;
    let exponent = (input & 0x3F) as i32;

    f16::from_f32(mantissa * 2f32.powi(exponent - 9))
}

/// Convert MIL-1750A hex (interpreted as u32) to f32
///
/// # Arguments
///
/// * `input`:  MIL-1750A hex (interpreted as u32)
///
/// returns: f32 representation of the input
///
/// # Examples
///
/// ```
/// use MIL1750A_Converter::m1750a_to_32flt;
/// assert_eq!(m1750a_to_32flt(0x997AE105), -25.6300010681152);
/// ```
pub fn m1750a_to_32flt(input: u32) -> f32 {
    let mantissa = (input >> 8) & 0xFFFFFF;
    let exponent = input & 0xFF;

    // Convert mantissa to signed two's complement
    let signed_mantissa = if mantissa & 0x800000 != 0 {
        -(((!mantissa & 0xFFFFFF) + 1) as i32)
    } else {
        mantissa as i32
    };

    (signed_mantissa as f32) * 2f32.powi((exponent as i32) - 23)
}

/// Convert MIL-1750A hex (interpreted as u64) to f48 (as f64)
///
/// # Arguments
///
/// * `input`:  MIL-1750A hex (interpreted as u64)
///
/// returns: f48 representation of the input
///
/// # Examples
///
/// ```
/// use MIL1750A_Converter::m1750a_to_48flt;
/// assert_eq!(m1750a_to_48flt(0x69A3B50754AB), 105.63948563742451);
/// ```
pub fn m1750a_to_48flt(input: u64) -> f64 {
    let mantissa1 = ((input >> 24) & 0xFFFFFF) as u32;
    let mantissa2 = (input & 0xFFFF) as u16;
    let exponent = ((input >> 16) & 0xFF) as i32;

    let value1 = (mantissa1 as f64) * 2f64.powi(exponent - 23);
    let value2 = (mantissa2 as f64) * 2f64.powi(exponent - 39);

    value1 + value2
}

#[cfg(test)]
mod tests {
    use super::*;
    use half::f16;

    #[test]
    fn test_f16_to_1750a() {
        assert_eq!(f16_to_1750a(f16::from_f32(-1.0)), 0x8000);
        assert_eq!(f16_to_1750a(f16::from_f32(1.0)), 0x8000);
        assert_eq!(f16_to_1750a(f16::from_f32(12.4)), 0x6344);
        assert_eq!(f16_to_1750a(f16::from_f32(-12.4)), 0x9CC4);
        assert_eq!(f16_to_1750a(f16::from_f32(25.63)), 0x6685);
        assert_eq!(f16_to_1750a(f16::from_f32(-25.63)), 0x9985);
    }

    #[test]
    fn test_f32_to_1750a() {
        assert_eq!(f32_to_1750a(1.0), 0x40000001);
        assert_eq!(f32_to_1750a(-1.0), 0x80000000);
        assert_eq!(f32_to_1750a(5.234), 0x53BE7703);
        assert_eq!(f32_to_1750a(-25.63f32), 0x997AE105);
        assert_eq!(f32_to_1750a(25.63f32), 0x66851F05);
    }

    #[test]
    fn test_f48_to_1750a() {
        assert_eq!(f48_to_1750a(105.639485637361), 0x69A3B50754AB);
        assert_eq!(f48_to_1750a(std::f64::consts::PI), 0x6487ED025111);
        assert_eq!(f48_to_1750a(-std::f64::consts::PI), 0x9B781202AEEF);
        assert_eq!(f48_to_1750a(-1.0), 0x800000_00_0000);
        assert_eq!(f48_to_1750a(0.0), 0x000000_00_0000);
    }

    #[test]
    fn test_m1750a_to_48flt() {
        assert_eq!(m1750a_to_48flt(0x69A3B50754AB), 105.63948563742451);
        assert_eq!(m1750a_to_48flt(0x64A3F4275AAB), 432247429803.0);
    }

    #[test]
    fn test_m1750a_to_32flt() {
        assert_eq!(m1750a_to_32flt(0x40000001), 1.0);
        assert_eq!(m1750a_to_32flt(0x997AE105), -25.6300010681152);
        assert_eq!(m1750a_to_32flt(0x9F34EA0C), -3097.3857421875);
    }

    #[test]
    fn test_m1750a_to_16flt() {
        assert_eq!(m1750a_to_16flt(0x6344), f16::from_f32(12.40625));
        assert_eq!(m1750a_to_16flt(0x324F), f16::from_f32(12864.0));
    }
}

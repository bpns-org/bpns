// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

pub fn format_sats(amount: u64) -> String {
    let mut number: String = format_number(amount as usize);
    number.push_str(" SAT");
    number
}

pub fn format_number(num: usize) -> String {
    let mut number: String = num.to_string();
    let number_len: usize = number.len();

    if number_len > 3 {
        let mut counter: u8 = 1;
        loop {
            if num / usize::pow(1000, counter.into()) > 0 {
                counter += 1;
            } else {
                break;
            }
        }

        counter -= 1;

        let mut formatted_number: Vec<String> =
            vec![number[0..(number_len - counter as usize * 3)].into()];

        number.replace_range(0..(number_len - counter as usize * 3), "");

        loop {
            if counter > 0 {
                if !number[0..3].is_empty() {
                    formatted_number.push(number[0..3].into());
                    number.replace_range(0..3, "");
                }

                counter -= 1
            } else {
                break;
            }
        }

        number = formatted_number.join(",");
    }

    number
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn format_num() {
        assert_eq!(format_number(180000), "180,000".to_string());
    }

    #[test]
    fn format_satoshi() {
        assert_eq!(format_sats(100), "100 SAT".to_string());
        assert_eq!(format_sats(1000), "1,000 SAT".to_string());
        assert_eq!(format_sats(10000), "10,000 SAT".to_string());
        assert_eq!(format_sats(100000), "100,000 SAT".to_string());
        assert_eq!(format_sats(1000000), "1,000,000 SAT".to_string());
        assert_eq!(format_sats(1000000000), "1,000,000,000 SAT".to_string());
    }
}

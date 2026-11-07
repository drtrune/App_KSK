#[cfg(test)]
mod tests {
    use crate::domain::scoring::*;
    use crate::domain::dates::*;

    #[test]
    fn test_age_calculation() {
        assert_eq!(is_valid_date(29, 2, 2024), true); // Leap year
        assert_eq!(is_valid_date(29, 2, 2023), false); // Non-leap year
        assert_eq!(parse_dmy("15/08/1995"), Some((15, 8, 1995)));
        assert_eq!(parse_dmy("invalid"), None);
    }

    #[test]
    fn test_blood_pressure() {
        // Under 30
        let (score1, _) = calc_blood_pressure_score("120/80", 25);
        assert_eq!(score1, 1);
        let (score2, _) = calc_blood_pressure_score("150/100", 25);
        assert_eq!(score2, 4);

        // Over 30
        let (score3, _) = calc_blood_pressure_score("130/85", 40);
        assert_eq!(score3, 1);
        let (score4, _) = calc_blood_pressure_score("142/92", 40);
        assert_eq!(score4, 3);
    }

    #[test]
    fn test_vision_scoring() {
        assert_eq!(calc_vision_score("10", "10"), 1);
        assert_eq!(calc_vision_score("9", "8"), 2);
        assert_eq!(calc_vision_score("7", "7"), 3);
        assert_eq!(calc_vision_score("5", "5"), 5);
    }

    #[test]
    fn test_hearing_scoring() {
        let (sc1, _) = calc_hearing_from_distances("5", "5", "5", "5");
        assert_eq!(sc1, 1);
        let (sc2, _) = calc_hearing_from_distances("4", "4", "2", "4");
        assert_eq!(sc2, 2);
    }
}


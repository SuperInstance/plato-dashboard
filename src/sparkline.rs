//! Sparkline: render numeric series as Unicode ASCII sparklines.

const SPARK_CHARS: &[char] = &['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

/// A sparkline renderer that converts a `Vec<f64>` into Unicode sparkline characters.
#[derive(Debug, Clone)]
pub struct Sparkline {
    values: Vec<f64>,
}

impl Sparkline {
    /// Create a new sparkline from a series of values.
    pub fn new(values: Vec<f64>) -> Self {
        Self { values }
    }

    /// Render the sparkline as a string of Unicode characters.
    /// Returns an empty string for empty input.
    pub fn render(&self) -> String {
        if self.values.is_empty() {
            return String::new();
        }
        if self.values.len() == 1 {
            return SPARK_CHARS[SPARK_CHARS.len() / 2].to_string();
        }

        let min = self.min();
        let max = self.max();
        let range = max - min;

        if range == 0.0 {
            return self.values.iter().map(|_| SPARK_CHARS[3]).collect();
        }

        self.values
            .iter()
            .map(|&v| {
                let normalized = (v - min) / range;
                let idx = (normalized * (SPARK_CHARS.len() - 1) as f64).round() as usize;
                let idx = idx.min(SPARK_CHARS.len() - 1);
                SPARK_CHARS[idx]
            })
            .collect()
    }

    /// Return the minimum value, or 0.0 if empty.
    pub fn min(&self) -> f64 {
        self.values.iter().cloned().fold(f64::INFINITY, f64::min)
    }

    /// Return the maximum value, or 0.0 if empty.
    pub fn max(&self) -> f64 {
        self.values.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
    }

    /// Return a trend arrow based on the last vs first value.
    /// ↑ rising, ↓ falling, → flat.
    pub fn trend_arrow(&self) -> char {
        if self.values.len() < 2 {
            return '→';
        }
        let first = self.values[0];
        let last = *self.values.last().unwrap();
        if last > first * 1.01 {
            '↑'
        } else if last < first * 0.99 {
            '↓'
        } else {
            '→'
        }
    }

    /// Render with min/max annotation appended.
    pub fn render_with_annotations(&self) -> String {
        if self.values.is_empty() {
            return String::new();
        }
        let spark = self.render();
        let trend = self.trend_arrow();
        format!("{} {} min:{:.1} max:{:.1} {}", spark, trend, self.min(), self.max(), trend)
    }

    /// Return the number of values in the series.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Check if the series is empty.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_simple_series() {
        let sp = Sparkline::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
        let rendered = sp.render();
        let chars: Vec<char> = rendered.chars().collect();
        assert_eq!(chars.len(), 8);
        assert_eq!(chars[0], '▁');
        assert_eq!(chars[7], '█');
    }

    #[test]
    fn test_render_empty_series() {
        let sp = Sparkline::new(vec![]);
        assert_eq!(sp.render(), "");
    }

    #[test]
    fn test_render_single_value() {
        let sp = Sparkline::new(vec![5.0]);
        // Single value maps to the middle character (index 4 = ▅)
        let rendered = sp.render();
        let ch: Vec<char> = rendered.chars().collect();
        assert_eq!(ch.len(), 1);
        assert_eq!(ch[0], SPARK_CHARS[SPARK_CHARS.len() / 2]);
    }

    #[test]
    fn test_min_max_annotation() {
        let sp = Sparkline::new(vec![2.0, 4.0, 6.0, 8.0]);
        let annotated = sp.render_with_annotations();
        assert!(annotated.contains("min:2.0"));
        assert!(annotated.contains("max:8.0"));
    }

    #[test]
    fn test_trend_arrow_rising() {
        let sp = Sparkline::new(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(sp.trend_arrow(), '↑');
    }

    #[test]
    fn test_trend_arrow_falling() {
        let sp = Sparkline::new(vec![10.0, 8.0, 6.0, 4.0, 2.0]);
        assert_eq!(sp.trend_arrow(), '↓');
    }

    #[test]
    fn test_trend_arrow_flat() {
        let sp = Sparkline::new(vec![5.0, 5.0, 5.0, 5.0]);
        assert_eq!(sp.trend_arrow(), '→');
    }

    #[test]
    fn test_trend_arrow_single_value() {
        let sp = Sparkline::new(vec![5.0]);
        assert_eq!(sp.trend_arrow(), '→');
    }

    #[test]
    fn test_all_same_values() {
        let sp = Sparkline::new(vec![3.0, 3.0, 3.0, 3.0]);
        let rendered = sp.render();
        assert!(rendered.chars().all(|c| c == '▄'));
    }

    #[test]
    fn test_len_and_is_empty() {
        let empty = Sparkline::new(vec![]);
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);

        let non_empty = Sparkline::new(vec![1.0]);
        assert!(!non_empty.is_empty());
        assert_eq!(non_empty.len(), 1);
    }
}

use crate::webp::types::WEBP_FILTER_TYPE;

const SMAX: i32 = 16;

// スコアリング差分を計算、[0..SMAX)の範囲
#[inline]
fn sdiff(a: u8, b: u8) -> i32 {
    ((a as i32 - b as i32).abs() >> 4)
}

// グラデーション予測器
#[inline]
fn gradient_predictor(a: u8, b: u8, c: u8) -> u8 {
    let g = a as i32 + b as i32 - c as i32;
    if g >= 0 && g <= 255 {
        g as u8
    } else if g < 0 {
        0
    } else {
        255
    }
}

/// 潜在的に興味深いフィルターモードの高速推定
pub fn estimate_best_filter(data: &[u8], width: i32, height: i32, stride: i32) -> WEBP_FILTER_TYPE {
    let mut bins = [[0i32; SMAX as usize]; WEBP_FILTER_TYPE::WEBP_FILTER_LAST as usize];

    // 2ピクセルごとにサンプリング
    for j in (2..height - 1).step_by(2) {
        let mut mean = data[(j * stride) as usize];
        for i in (2..width - 1).step_by(2) {
            let pos = (j * stride + i) as usize;
            let p_i = data[pos];
            let p_i_prev = data[pos - 1];
            let p_i_top = data[pos - stride as usize];
            let p_i_top_left = data[pos - stride as usize - 1];

            let diff0 = sdiff(p_i, mean);
            let diff1 = sdiff(p_i, p_i_prev);
            let diff2 = sdiff(p_i, p_i_top);
            let grad_pred = gradient_predictor(p_i_prev, p_i_top, p_i_top_left);
            let diff3 = sdiff(p_i, grad_pred);

            bins[WEBP_FILTER_TYPE::WEBP_FILTER_NONE as usize][diff0 as usize] = 1;
            bins[WEBP_FILTER_TYPE::WEBP_FILTER_HORIZONTAL as usize][diff1 as usize] = 1;
            bins[WEBP_FILTER_TYPE::WEBP_FILTER_VERTICAL as usize][diff2 as usize] = 1;
            bins[WEBP_FILTER_TYPE::WEBP_FILTER_GRADIENT as usize][diff3 as usize] = 1;

            mean = ((3 * mean as i32 + p_i as i32 + 2) >> 2) as u8;
        }
    }

    // 最良のフィルターを見つける
    let mut best_score = i32::MAX;
    let mut best_filter = WEBP_FILTER_TYPE::WEBP_FILTER_NONE;

    for filter in 0..WEBP_FILTER_TYPE::WEBP_FILTER_LAST as usize {
        let score: i32 = (0..SMAX as usize)
            .filter(|&i| bins[filter][i] > 0)
            .map(|i| i as i32)
            .sum();

        if score < best_score {
            best_score = score;
            best_filter = WEBP_FILTER_TYPE::from(filter as u32);
        }
    }

    best_filter
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gradient_predictor() {
        assert_eq!(gradient_predictor(100, 100, 100), 100);
        assert_eq!(gradient_predictor(255, 255, 255), 255);
        assert_eq!(gradient_predictor(0, 0, 0), 0);
        assert_eq!(gradient_predictor(100, 150, 200), 50);
    }

    #[test]
    fn test_sdiff() {
        assert_eq!(sdiff(16, 0), 1);
        assert_eq!(sdiff(0, 16), 1);
        assert_eq!(sdiff(255, 255), 0);
        assert_eq!(sdiff(0, 0), 0);
    }
}

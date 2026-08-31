//! 文件名自然排序
//!
//! 本地下载目录重建图片列表时用：图片文件名是服务端 `originalName`
//! （如 `15_08.jpg`）或下载器补的序号名（如 `0001.jpg`），也可能出现
//! 无前导零的 `1.jpg`/`10.jpg`——纯字典序会把 `10` 排到 `2` 前面，
//! 页序全乱。自然排序把连续数字段按数值比较，其余按字符比较。

use std::cmp::Ordering;

/// 自然排序比较：数字段按数值、非数字段按字符
///
/// 数字比较不解析成整数（避免超长数字溢出）：跳过前导零后先比位数、
/// 再逐位比较。数值相等但前导零不同（`01` vs `1`）时按原始段长打破平局，
/// 保证全序稳定。
pub fn natural_cmp(a: &str, b: &str) -> Ordering {
    let mut ia = a.chars().peekable();
    let mut ib = b.chars().peekable();

    loop {
        match (ia.peek().copied(), ib.peek().copied()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(ca), Some(cb)) => {
                if ca.is_ascii_digit() && cb.is_ascii_digit() {
                    // 各取一段连续数字
                    let da = take_digits(&mut ia);
                    let db = take_digits(&mut ib);
                    let ord = cmp_digit_runs(&da, &db);
                    if ord != Ordering::Equal {
                        return ord;
                    }
                } else {
                    if ca != cb {
                        return ca.cmp(&cb);
                    }
                    ia.next();
                    ib.next();
                }
            }
        }
    }
}

/// 取出迭代器头部的连续数字段
fn take_digits(iter: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    let mut digits = String::new();
    while let Some(&c) = iter.peek() {
        if !c.is_ascii_digit() {
            break;
        }
        digits.push(c);
        iter.next();
    }
    digits
}

/// 比较两段数字：跳前导零 → 比位数 → 逐位比 → 段长打破平局
fn cmp_digit_runs(a: &str, b: &str) -> Ordering {
    let sa = a.trim_start_matches('0');
    let sb = b.trim_start_matches('0');
    sa.len()
        .cmp(&sb.len())
        .then_with(|| sa.cmp(sb))
        .then_with(|| a.len().cmp(&b.len()))
}

#[cfg(test)]
mod tests {
    use super::natural_cmp;

    /// 无前导零的纯数字名：字典序会排出 1,10,2，自然排序必须是 1,2,10
    #[test]
    fn numeric_names_sort_by_value() {
        let mut names = vec!["10.jpg", "2.jpg", "1.jpg"];
        names.sort_by(|a, b| natural_cmp(a, b));
        assert_eq!(names, ["1.jpg", "2.jpg", "10.jpg"]);
    }

    /// 实际下载样本形态：`{章}_{页}` 零填充
    #[test]
    fn underscore_padded_names_keep_page_order() {
        let mut names = vec!["15_32.jpg", "15_08.jpg", "15_18.jpg"];
        names.sort_by(|a, b| natural_cmp(a, b));
        assert_eq!(names, ["15_08.jpg", "15_18.jpg", "15_32.jpg"]);
    }

    /// 下载器补号形态：四位零填充
    #[test]
    fn zero_padded_sequence() {
        let mut names = vec!["0010.jpg", "0002.jpg", "0001.jpg"];
        names.sort_by(|a, b| natural_cmp(a, b));
        assert_eq!(names, ["0001.jpg", "0002.jpg", "0010.jpg"]);
    }

    /// 字母前缀 + 数字混合
    #[test]
    fn mixed_alpha_numeric() {
        let mut names = vec!["p10.png", "p2.png", "p1.png", "cover.png"];
        names.sort_by(|a, b| natural_cmp(a, b));
        assert_eq!(names, ["cover.png", "p1.png", "p2.png", "p10.png"]);
    }

    /// 前导零数值相等时按段长打破平局，保证全序确定
    #[test]
    fn leading_zero_tiebreak_is_stable() {
        assert_eq!(natural_cmp("01.jpg", "1.jpg"), std::cmp::Ordering::Greater);
        assert_eq!(natural_cmp("1.jpg", "1.jpg"), std::cmp::Ordering::Equal);
    }

    /// 一边是另一边的前缀
    #[test]
    fn prefix_relationship() {
        assert_eq!(natural_cmp("1.jpg", "1_2.jpg"), std::cmp::Ordering::Less);
    }
}

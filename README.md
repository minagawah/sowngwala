# sowngwala

## About

__"sowngwala"__ is a library for calculating sun's position.
__"sowng-"__ stands for the _"sun"_ in
_[Belter language](https://expanse.fandom.com/wiki/Belter_Creole)_
_(from a Sci-Fi TV series "The Expanse" (2015))_,
and __"-wala"__ for _"one who is professional at"_.

Most of the materials provided in the program are based on:

- [Peter Duffett-Smith "Practical Astronomy With Your Calculator"  
(The Press Syndicate of the University of Cambridge, 1988)](https://books.google.co.jp/books?id=DwJfCtzaVvYC&hl=ja&source=gbs_book_similarbooks)

## Examples

### (Example 1) Finding Julian Day

On Saturday, October 26, 1985, 1:35 AM,
Marty McFly departs to 1955.
Let's see if we can find the julian day for the date.

```rust
use chrono::naive::{NaiveDate, NaiveDateTime};
use crate::time::julian_day_from_generic_datetime;

let datetime: NaiveDateTime = NaiveDate::from_ymd(1985, 10, 26)
    .and_hms(1, 35, 0);

let jd: f64 = julian_day_from_generic_datetime(datetime);
// jd: 2446364.565972222
```

### (Example 2) Finding "Zhi"

In Chinese astrology, a year is divided into 12, called "支" (Zhi), or _"Branches"_,
and you can easily figure out by finding the ecliptic longitude of the sun.

Find more on Chinese astrology in
_["mikaboshi"](https://github.com/minagawah/mikaboshi)_,
a Feng-Shui calculation library using _"sowngwala"_.

```rust
use chrono::naive::NaiveDate;
use sowngwala::coords::EcliCoord;
use sowngwala::sun::ecliptic_position_of_the_sun_from_generic_date;

let date: NaiveDate = NaiveDate::from_ymd(2022, 5, 6);

let ecliptic: EcliCoord =
    ecliptic_position_of_the_sun_from_generic_date(date);

let lng: f64 = ecliptic.lng;

// Monthly Zhi should result in `3` for 5/6/2022.
let branch: usize = if (315.0..345.0).contains(&lng) {
    0 // 立春 (lichun) + 雨水 (yushui) ---> 寅 (yin)
} else if !(15.0..345.0).contains(&lng) {
    1 // 啓蟄 (jingzhe) + 春分 (chunfen) ---> 卯 (mao)
} else if (15.0..45.0).contains(&lng) {
    2 // 清明 (qingming) + 穀雨 (guyu) ---> 辰 (chen)
} else if (45.0..75.0).contains(&lng) {
    3 // 立夏 (lixia) + 小滿 (xiaoman) ---> 巳 (si)
} else if (75.0..105.0).contains(&lng) {
    4 // 芒種 (mangzhong) + 夏至 (xiazhi) ---> 午 (wu)
} else if (105.0..135.0).contains(&lng) {
    5 // 小暑 (xiaoshu) + 大暑 (dashu) ---> 未 (wei)
} else if (135.0..165.0).contains(&lng) {
    6 // 立秋 (liqiu) + 處暑 (chushu) ---> 申 (shen)
} else if (165.0..195.0).contains(&lng) {
    7 // 白露 (bailu) + 秋分 (qiufen) ---> 酉 (you)
} else if (195.0..225.0).contains(&lng) {
    8 // 寒露 (hanlu) + 霜降 (shuangjiang) ---> 戌 (xu)
} else if (225.0..255.0).contains(&lng) {
    9 // 立冬 (lidong) + 小雪 (xiaoxue) ---> 亥 (hai)
} else if (255.0..285.0).contains(&lng) {
    10 // 大雪 (daxue) + 冬至 (dongzhi) ---> 子 (zi)
} else {
    // lng >= 285.0 || lng < 315.0
    11 // 小寒 (xiaohan) + 大寒 (dahan) ---> 丑 (chou)
};
```

## Dislaimer

There is absolutely no gurantee about the accuracy of the service,
information, or calculated results provided by the program,
and the author of the program cannot be held responsible
in any ways for any adverse consequences.
It is solely for entertaniment only, and your use of the service,
information, or calculated results is entirely at your own risks,
for which the author of the program shall not be liable.
It shall be your own responsibility to ensure the service,
information, or calculated results meet your specific requirements.

## License

MIT license ([LICENSE](LICENSE))

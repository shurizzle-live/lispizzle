### skip_characters.rs

On an old macbook pro mid 2014 (i5-4278U CPU @ 2.60GHz)

```
count/algo              time:   [36.730 µs 36.834 µs 36.942 µs]
                        change: [-1.9977% -0.7712% +0.5393%] (p = 0.24 > 0.05)
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) low mild
  3 (3.00%) high severe
count/naive             time:   [32.005 µs 32.241 µs 32.480 µs]
                        change: [+2.5186% +3.8387% +5.1992%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe

skip 100/algo           time:   [43.974 ns 44.040 ns 44.117 ns]
                        change: [-4.1476% -2.6040% -1.2296%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
skip 100/naive          time:   [115.07 ns 115.22 ns 115.40 ns]
                        change: [-0.3677% +0.1982% +0.8205%] (p = 0.52 > 0.05)
                        No change in performance detected.
Found 13 outliers among 100 measurements (13.00%)
  5 (5.00%) high mild
  8 (8.00%) high severe

skip 300_000/algo       time:   [99.929 µs 100.30 µs 100.66 µs]
                        change: [-1.0451% -0.3271% +0.3713%] (p = 0.36 > 0.05)
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
skip 300_000/naive      time:   [295.68 µs 296.22 µs 296.79 µs]
                        change: [-0.6623% +0.1368% +0.9684%] (p = 0.75 > 0.05)
                        No change in performance detected.
Found 10 outliers among 100 measurements (10.00%)
  6 (6.00%) high mild
  4 (4.00%) high severe

skip-count 100/algo     time:   [116.91 ns 117.08 ns 117.26 ns]
                        change: [-0.5431% -0.0638% +0.4506%] (p = 0.80 > 0.05)
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
skip-count 100/naive    time:   [162.20 ns 162.53 ns 162.92 ns]
                        change: [-0.6554% -0.0318% +0.5353%] (p = 0.93 > 0.05)
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe

skip-count 300_000/algo time:   [311.93 µs 312.33 µs 312.78 µs]
                        change: [-16.711% -11.266% -6.4099%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 13 outliers among 100 measurements (13.00%)
  9 (9.00%) high mild
  4 (4.00%) high severe
skip-count 300_000/naive
                        time:   [440.64 µs 441.73 µs 443.39 µs]
                        change: [-0.4233% +0.0576% +0.5536%] (p = 0.84 > 0.05)
                        No change in performance detected.
Found 9 outliers among 100 measurements (9.00%)
  2 (2.00%) high mild
  7 (7.00%) high severe
```

On Linux i7-10750H CPU @ 2.60GHz

```
count/algo              time:   [18.172 µs 18.217 µs 18.268 µs]
                        change: [-0.2383% +0.4597% +1.0863%] (p = 0.19 > 0.05)
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) high mild
  4 (4.00%) high severe
count/naive             time:   [18.612 µs 18.672 µs 18.738 µs]
                        change: [-0.6951% -0.0311% +0.6689%] (p = 0.93 > 0.05)
                        No change in performance detected.
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe

skip 100/algo           time:   [26.727 ns 26.827 ns 26.947 ns]
                        change: [-0.6610% +0.1060% +0.8938%] (p = 0.79 > 0.05)
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe
skip 100/naive          time:   [115.56 ns 115.79 ns 116.06 ns]
                        change: [-1.0042% -0.5098% +0.0242%] (p = 0.05 > 0.05)
                        No change in performance detected.
Found 11 outliers among 100 measurements (11.00%)
  7 (7.00%) high mild
  4 (4.00%) high severe

skip 300_000/algo       time:   [62.093 µs 62.304 µs 62.525 µs]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
skip 300_000/naive      time:   [322.57 µs 323.23 µs 323.99 µs]
Found 13 outliers among 100 measurements (13.00%)
  9 (9.00%) high mild
  4 (4.00%) high severe

skip-count 100/algo     time:   [67.430 ns 67.814 ns 68.270 ns]
                        change: [-5.7140% -5.0082% -4.2352%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe
skip-count 100/naive    time:   [127.10 ns 127.32 ns 127.55 ns]
                        change: [-2.6021% -2.0781% -1.5958%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe

skip-count 300_000/algo time:   [181.18 µs 181.63 µs 182.16 µs]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
skip-count 300_000/naive
                        time:   [346.31 µs 347.16 µs 348.03 µs]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
```

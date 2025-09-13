# Wax vs ignore test

We saw that the performance was still pretty bad on projects with .pixi folders for wax.
Because we previously concluded that ignore was traversing into more directories than wax.
This all seemed a bit strange.
I setup a quick benchmark to assess with a large pixi folder what happens and I see a wax taken ~=100x as long as ignore
for this example.

The test is that I include a number of globs that we typically use, and only ignore all hidden folders for both wax and ignore.

An example run on my M1 PRO macbook:

```
wax_total               time:   [372.56 ms 373.45 ms 374.44 ms]
                        change: [-1.0384% -0.5491% -0.0880%] (p = 0.03 < 0.05)
                        Change within noise threshold.
Found 15 outliers among 100 measurements (15.00%)
  5 (5.00%) high mild
  10 (10.00%) high severe

ignore_total            time:   [4.7775 ms 4.7876 ms 4.7998 ms]
                        change: [+3.7295% +4.1447% +4.5423%] (p = 0.00 < 0.05)
                        Performance has regressed.
```

Here you can see the results, I still think 4ms is long for ignore but turning on all default filters seems to get it back to about 1ms.

## Test this yourself

1. Make sure you have pixi.
2. `pixi r bench` should compile and run the sanity checks plus the bench. This will also fill up the .pixi folder with a lot of stuff, filling it with more stuffs seems to make wax slower.

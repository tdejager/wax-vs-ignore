# Wax vs ignore test

We saw that the performance was still pretty bad on projects with .pixi folders for wax.
Because we previously concluded that ignore was traversing into more directories than wax.
This all seemed a bit strange.
I setup a quick benchmark to assess with a large pixi folder what happens and I see a wax taken 100x as long as ignore
for this example.

## Test this yourself

1. Make sure you have pixi.
2. `pixi r bench` should compile and run the sanity checks plus the bench

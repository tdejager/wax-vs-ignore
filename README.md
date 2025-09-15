# Wax vs ignore (and other globs) test

We saw that the runtime of glob parsing and traversing was pretty high for projects with .pixi folders for wax.
Because we previously concluded that ignore was traversing into more directories than wax.
This all seemed a bit strange.
I setup a quick benchmark to assess with a large pixi folder what happens and I see a wax taken ~=100x as long as ignore
for this example.

The test is that I include a number of globs that we typically use, and only ignore all hidden folders for both wax and ignore.

These are the globs we test against:

```
    "**/*.{c,cc,cxx,cpp,h,hpp,hxx}",
    "**/*.{cmake,cmake.in}",
    "**/CMakeFiles.txt",
```

For wax we have a `not` expression to filter out negative matches and a `hidden(true)` for ignore.
And we use the "." as the root.
This project also contains a number of dummy files so that there are at least some matches.

**Note, I found out that on the main branch of wax, things are much improved**

Still, there seem to be some differences but it does not seem to traverse into hidden directories at the root level at least.

<img width="1201" height="228" alt="image" src="https://github.com/user-attachments/assets/b4adc334-05a9-452d-a1df-fadb62ee2e69" />


## Test this yourself

1. Make sure you have pixi. (`curl -fsSL https://pixi.sh/install.sh | sh`)
2. `pixi r bench` should compile and run the sanity checks plus the bench. This will also fill up the .pixi folder with a lot of stuff, filling it with more stuffs seems to make wax slower.


## To use a bigger example
Run `pixi r opencv-src` to get the opencv source.
Add it to your `.git/info/exclude` file to not have it interfere when testing `.gitignore` with different glob libraries.

<img width="752" height="140" alt="image" src="https://github.com/user-attachments/assets/9bc0298f-842d-4def-b0c9-acfb5bbb2433" />


# Clunky
An awkward game framework.

## Purpose:
Clunky is designed to be relatively simple to use, while maintaining decent performance. It should be extremely flexible.

It features 2 different ways of rendering with vulkan (although it shouldn't be too difficult to modify it to your heart's content):
1. By setting some enums you can specify what render buffers to use and a couple other rendering settings. This isn't very flexible, and is also quite slow due to the many match statements that have to be used by the code interpreting the enums. This is very simple and easy to work with though. It might also require less copying and pasting, if you don't plan to add many vertex and instance types.
2. You can manually interact with the draw call builder to do anything you want. This is extremely flexible, and very fast. This does require you to handle your buffers, pipelines, and other rendering stuff, yourself though, which is more complicated (although still fairly simple), but may increase copying and pasting when multiple menus have the same rendering scheme.

## Getting Started:
We have a couple example menus, but otherwise good luck. Tutorials and stuff will be added soon though.

## Getting Help:
TODO

## Contributing:
This is a real mess, and I hardly know how to use github, so any and all contribution is welcome, I would love to turn this into a proper open source project with many people contributing one day.

# Clunky
An awkward game framework.

## Purpose:
Clunky is designed to be relatively simple to use, while maintaining decent performance. It should be extremely flexible. I also want it to have a wide range of features, such as physics, premade shaders, mesh loading, scene loading, and more.

## Features
It is still very early days, and as such there are very few features.
It currently has 2 sides, user code which you edit, and main code which you usually don't touch as much. Each side has its own struct (UserStorage and RenderStorage). User code is allowed to read and alter render_storage, but main should never interact with user_storage.

It currently features 2 different ways of rendering with vulkan (although it shouldn't be too difficult to modify it to your heart's content):
1. By setting some enums you can specify what render buffers to use and a couple other rendering settings. This isn't very flexible, and is also quite slow due to the many match statements that have to be used by the code interpreting the enums. This is very simple and easy to work with though. It might also require less copying and pasting, if you don't plan to add many vertex and instance types.
2. You can manually interact with the draw call builder to do anything you want. This is extremely flexible, and very fast. This does require you to handle your buffers, pipelines, and other rendering stuff, yourself though, which is more complicated (although still fairly simple), but may increase copying and pasting when multiple menus have the same rendering scheme.

In terms of physics we currently have a very barebones aabb and verlet integration physics system that runs single threaded on the cpu, and is very temporary.

Mesh loading and scene loading sort of works from gltf, but gltf is +y up, and by default Clunky is -y up. We can rotation by 180 degrees in the x or z directions, but the terrain is still horizontally mirrored then, which is unfortunate. See issue INSERT ISSUE HERE ASAP.

## Getting Started:
We have a couple example menus, but otherwise good luck. Tutorials and stuff will be added soon though.

## Getting Help:
Feel free to ask a question in [discussions](https://github.com/coolcatcoder/Clunky/discussions), or raise a problem in [issues](https://github.com/coolcatcoder/Clunky/issues).
We don't have any specific discord server, but [my discord server](https://discord.gg/43yfpHxVrz) can be used in the meantime.

## Contributing:
This is a real mess, and I hardly know how to use github, so any and all contribution is welcome, I would love to turn this into a proper open source project with many people contributing one day. Note that there are a lot of issues, yet very few will be in github issues, because I'm lazy and also the only one who knows that Clunky exists, so creating issues just wastes time I could be spending fixing problems.

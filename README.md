paths
-----

*paths* is a software path-tracing renderer written in Rust.

It is purely a hobby-project for me to learn about this technology, and I had no idea how path-tracing worked (other than vague intuition) before starting.


### Implementation Checklist

- [x] Basic path tracing of spheres
- [x] Point lights
- [x] Multithreading
- [x] Collision acceleration (BVH)
- [ ] Textures
- [x] Triangle meshes
- [x] Normal smoothing (of triangle meshes)
- [x] Importance sampling
- [x] Reflective materials
- [x] Glossy materials
- [x] Microfacet specular model
- [x] Camera lens simulation
- [x] Next event estimation
- [x] HDRI Environment maps
- [ ] Translucent materials (refraction)
- [ ] Subsurface scattering
- [ ] Bi-directional path tracing

### Examples

[2020/05/23] Support for HDRI environment maps
(thanks for [HDRI Haven](https://hdrihaven.com/) for the HDRIs!)
![image](https://user-images.githubusercontent.com/3620166/82724590-1ecdd700-9d12-11ea-8814-d0010b949349.png)

[2020/05/15] Support for per-vertex colours baked into meshes
(thanks to [Artec3D](https://www.artec3d.com/3d-models/doom-combat-scene) for this awesome model):
![image](https://user-images.githubusercontent.com/3620166/81828266-92cee900-9574-11ea-8d76-28f2874c82e8.png)

[2020/05/10] Rewritten large parts of the renderer so that the results match a reference renderer ([Mitsuba](https://www.mitsuba-renderer.org/))
![image](https://user-images.githubusercontent.com/3620166/81772760-a1d87b80-9521-11ea-85af-59718edd15cc.png)
![image](https://user-images.githubusercontent.com/3620166/81502110-ef34cd00-9316-11ea-8d4b-e122d8d6d901.png)

[2020/05/04] Next event estimation to efficiently simulate point/area light sources
![image](https://user-images.githubusercontent.com/3620166/80952676-01090280-8e35-11ea-9580-db4ec2e045bc.png)

[2020/04/22] Microfacet model to create shiny, rough teapot material
![image](https://user-images.githubusercontent.com/3620166/79968000-87832300-84ca-11ea-9807-384fef0dbc7e.png)

[2019/03/14] Teapot model with ~6000 triangles and smoothed normals
![image](https://user-images.githubusercontent.com/3620166/54364159-a603d180-46af-11e9-973c-cbab9fac9685.png)

[2019/03/11] 500 randomly generated spheres
![image](https://user-images.githubusercontent.com/3620166/54086894-1b5e6200-4391-11e9-8400-041ce5de0579.png)

[2019/03/07] Close up of green sphere with strong bokeh on the background
![image](https://user-images.githubusercontent.com/3620166/53971014-579a8400-413f-11e9-9bf7-3c5932cb6df1.png)

[2019/03/05] 2 glossy spheres and one perfect mirror sphere on a reflective plane.
![image](https://user-images.githubusercontent.com/3620166/53858421-7ef93000-401d-11e9-9356-31258a0367bd.png)

[2019/03/03] 2 Spheres with large off-screen light source
![image](https://user-images.githubusercontent.com/3620166/53704391-b9b56980-3e5f-11e9-8a36-eb9baaf8630a.png)

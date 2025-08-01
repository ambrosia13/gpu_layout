# gpu_layout

Share data between CPU host code and GPU shader code, making graphics programs much more maintainable and scalable. No more need for `#[repr(C)]` and manual padding bytes!

Both the `std140` and `std430` layouts are implemented, and creating a custom layout is as easy as implementing the `GpuLayout` trait.

## why not use bytemuck / raw transmuting?

A very efficient approach is to just cast your structs directly to slices of bytes, like so:

```rs
#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
struct Test {
    a: Vec3,
    b: f32,
}

let my_data: Test = todo!();
let my_data: &[u8] = bytemuck::bytes_of(&my_data);
```

This requires no data copying, and thus is very memory efficient and performant. However, it has a couple drawbacks:

- forcing `#[repr(C)]`, meaning the order of your struct fields greatly matters for its layout and size
- you will likely have to insert manual padding bytes. the following will not work with GPU layout rules:

```rs
// Doesn't work: padding bytes are required
#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
struct TwoVec3s {
    a: Vec3,
    b: Vec3,
}

// CPU Vec3: size = 12, alignment = 4
// CPU size: 12 + 12 = 24
// CPU alignment: 4
//
// GPU vec3: size = 12, alignment = 16
// GPU size: 12 + 4 (padding) + 12 + 4 (padding) = 32
// GPU alignment: 16
```

Instead, you must insert manual padding:

```rs
#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
struct TwoVec3s {
    a: Vec3,
    _padding0: u32,
    b: Vec3,
    _padding1: u32,
}
```

As you begin to transfer large amounts of data from the CPU to the GPU, this quickly becomes unmaintainable. For example, try to imagine where you would place padding to format the following struct correctly:

```rs
pub struct CameraUniform {
    view_projection_matrix: Mat4,
    view_matrix: Mat4,
    projection_matrix: Mat4,

    inverse_view_projection_matrix: Mat4,
    inverse_view_matrix: Mat4,
    inverse_projection_matrix: Mat4,

    previous_view_projection_matrix: Mat4,
    previous_view_matrix: Mat4,
    previous_projection_matrix: Mat4,

    position: Vec3,
    previous_position: Vec3,

    view: Vec3,
    previous_view: Vec3,

    right: Vec3,
    up: Vec3,
}
```

This crate solves this problem by handling the padding automatically, at the cost of having to make a copy of the CPU data to format it for the GPU.

## example

### automatic implementation

If your struct consists only of the following:
- GPU primitives (`f32`/`float`, `i32`/`int`, `u32`/`uint`, `floatN`/`vecN`, `intN`/`ivecN`, `uintN`/`uvecN`, `floatNxN`/`matN`, etc.)
    - currently, only `glam` is implemented as representations of vector and matrix types
    - currently, only matrix types with the same number of rows and columns are implemented
- an array comprising of those primitives (`Vec<T>`, `&[T]`, or `[T; N]`)
- any type that implements the trait `AsGpuBytes`

then you can derive this crate's main trait, `AsGpuBytes`, for your struct, and get a slice of bytes that you can pass to the GPU without any further processing.

```rs
#[derive(AsGpuBytes)]
struct TestA {
    a: UVec3,
    b: u32,
}

#[derive(AsGpuBytes)]
struct TestB {
    a: Vec<u32>,
}

let a = TestA {
    a: UVec3::splat(u32::MAX),
    b: u32::MAX,
};

let b = TestB {
    a: vec![u32::MAX, u32::MAX],
};

// insert and align individual fields of the struct
let mut a = a.as_gpu_bytes::<Std430Layout>();
// align the struct and get a slice view of it, when you're ready to write it to a GPU buffer
let a: &[u8] = a.as_slice(); 

// uvec3: size 12, alignment 16
// uint: size 4, alignment 4
// result: size 16, alignment 16
assert_eq!(a.len(), 16);

let mut b = b.as_gpu_bytes::<Std140Layout>();
let b: &[u8] = b.as_slice();

// in std140, array elements are aligned to 16 bytes
assert_eq!(b.len(), 32);
```

> Note: `Cow<'_, [u8]>` is used to minimize how much data is copied in the intermediate steps. Copying is only avoided entirely with `GpuBytes::from_slice`, which is used with GPU primitives.

### manual implementation

If your CPU-side struct definition doesn't match how the GPU should interpret the data, for whatever reason, you can also implement the conversion manually:

```rs
#[derive(AsGpuBytes)]
struct Sphere {
    position: Vec3,
    radius: f32,
}

struct SphereList {
    spheres: Vec<Sphere>,
}

impl AsGpuBytes for struct SphereList {
    fn as_gpu_bytes<L: GpuLayout + ?Sized>(&self) -> GpuBytes {
        let mut buf = GpuBytes::empty();

        // GPU equivalent:
        // 
        // struct SphereList {
        //     uint count;
        //     Sphere[] spheres;
        // }

        buf.write(&(self.data.len() as u32))
           .write(&self.data);

        buf
    }
}
```

## usage

example for usage with `wgpu`, using the structs in the previous example:

```rs 
let sphere_list = SphereList {
    spheres: vec![
        Sphere { position: Vec3::splat(0.0), radius: 0.5 },
        Sphere { position: Vec3::splat(5.0), radius: 4.0 }, 
    ],
};

let sphere_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    label: None,
    contents: sphere_list.as_gpu_bytes::<Std430Layout>().as_slice(),
    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
});
```
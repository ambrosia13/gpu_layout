# gpu_layout

Share data between CPU host code and GPU shader code, making it much easier to write graphics programs using `wgpu`, Vulkan, or another graphics API. No more need for `#[repr(C)]` and manual padding bytes!

Both the `std140` and `std430` layouts are implemented, and if you have another layout you would like to implement, you can do so by implementing the `GpuLayout` trait.

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
// align the whole struct and get it as a slice, only call this if you are finished converting all your data; if you are including this in another struct, don't call this as it'll add unneeded padding
let a: &[u8] = a.as_slice(); 

// vec3 has a size of 12 bytes but an alignment of 16 bytes,
// this crate respects that rule, so you can pack a float
// right after a vec3 and take no extra space
assert_eq!(a.len(), 16);

let mut b = b.as_gpu_bytes::<Std140Layout>();
let b: &[u8] = b.as_slice();

// in std140, array elements are aligned to 16 bytes
assert_eq!(b.len(), 32);
```

> Note: `Cow<'_, u8>` is used to minimize how much data is copied in the intermediate steps. Copying is only avoided entirely with `GpuBytes::from_slice`, which is used with GPU primitives.

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
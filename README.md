# gpu_layout

Share data between CPU host code and GPU shader code, making it much easier to write graphics programs using `wgpu`, Vulkan, or another graphics API. No more need for `#[repr(C)]` and manual padding bytes!

Both the `std140` and `std430` layouts are implemented, and if you have another layout you would like to implement, such as the scalar data layout in Vulkan, you can do so by implementing the `GpuLayout` trait.

## example

### automatic implementation

If your struct consists only of the following:
- GPU primitives (`f32`/`float`, `i32`/`int`, `u32`/`uint`, `floatN`/`vecN`, `intN`/`ivecN`, `uintN`/`uvecN`, `floatNxN`/`matN`, etc.)
    - currently, only `glam` is implemented as representations of vector and matrix types
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

let a = a.as_gpu_bytes::<Std430Layout>(); // align and insert individual fields
let a: Cow<'_, u8> = Std430Layout::finish(a); // align the whole struct

// vec3 has a size of 12 bytes but an alignment of 16 bytes,
// this crate respects that rule, so you can pack a float
// right after a vec3 and take no extra space
assert_eq!(a.len(), 16);

let b = b.as_gpu_bytes::<Std140Layout>();
let b: Cow<'_, u8> = Std140Layout::finish(b);

// in std140, array elements are aligned to 16 bytes
assert_eq!(b.len(), 32);
```

> Note: `Cow<'_, u8>` is used to minimize how much data is copied in the intermediate steps. Copying is only avoided entirely with `GpuBytes::from_slice`, which is used with GPU primitives.

### manual implementation

If your CPU-side struct definition doesn't match how the GPU should interpret the data, for whatever reason, you can also implement the conversion manually:

```rs
struct TestC {
    data: Vec<Vec3>,
}

impl AsGpuBytes for TestC {
    fn as_gpu_bytes<L: GpuLayout + ?Sized>(&self) -> GpuBytes {
        let mut buf = GpuBytes::empty();

        // GPU equivalent:
        // struct TestC {
        //     uint count;
        //     float3[] data;
        // }

        L::write(&mut buf, &(self.data.len() as u32));
        L::write(&mut buf, &self.data);

        buf
    }
}
```
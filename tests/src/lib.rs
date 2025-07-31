#[cfg(test)]
mod tests {
    use glam::{UVec3, Vec3};
    use gpu_layout::{AsGpuBytes, GpuBytes, GpuLayout, Std140Layout, Std430Layout};

    #[test]
    fn f32_and_vec3() {
        let mut buf = GpuBytes::empty();

        Std140Layout::write(&mut buf, &UVec3::splat(u32::MAX));
        Std140Layout::write(&mut buf, &u32::MAX);

        let bytes = Std140Layout::finish(buf);

        #[rustfmt::skip]
        assert_eq!(
            &*bytes,
            &[
                // x
                u8::MAX, u8::MAX, u8::MAX, u8::MAX,
                // y
                u8::MAX, u8::MAX, u8::MAX, u8::MAX,
                // z
                u8::MAX, u8::MAX, u8::MAX, u8::MAX,
                // scalar
                u8::MAX, u8::MAX, u8::MAX, u8::MAX,
            ]
        );
    }

    #[test]
    fn vec3_and_vec3() {
        let mut buf = GpuBytes::empty();

        Std140Layout::write(&mut buf, &UVec3::splat(u32::MAX));
        Std140Layout::write(&mut buf, &UVec3::splat(u32::MAX));

        let bytes = Std140Layout::finish(buf);

        #[rustfmt::skip]
        assert_eq!(
            &*bytes,
            &[
                // x
                u8::MAX, u8::MAX, u8::MAX, u8::MAX,
                // y
                u8::MAX, u8::MAX, u8::MAX, u8::MAX,
                // z
                u8::MAX, u8::MAX, u8::MAX, u8::MAX,

                // padding
                0, 0, 0, 0,

                // x
                u8::MAX, u8::MAX, u8::MAX, u8::MAX,
                // y
                u8::MAX, u8::MAX, u8::MAX, u8::MAX,
                // z
                u8::MAX, u8::MAX, u8::MAX, u8::MAX,

                // padding
                0, 0, 0, 0

            ]
        );
    }

    #[test]
    fn std140_scalar_array() {
        let mut buf = GpuBytes::empty();

        Std140Layout::write_array(&mut buf, &[u32::MAX, u32::MAX]);

        let bytes = Std140Layout::finish(buf);

        #[rustfmt::skip]
        assert_eq!(
            &*bytes,
            &[
                // scalar
                u8::MAX, u8::MAX, u8::MAX, u8::MAX,
                // padding
                0, 0, 0, 0, 
                // padding
                0, 0, 0, 0, 
                // padding
                0, 0, 0, 0, 
                // scalar
                u8::MAX, u8::MAX, u8::MAX, u8::MAX,
                // padding
                0, 0, 0, 0, 
                // padding
                0, 0, 0, 0, 
                // padding
                0, 0, 0, 0, 
            ]
        );
    }

    #[test]
    fn std430_scalar_array() {
        let mut buf = GpuBytes::empty();

        Std430Layout::write_array(&mut buf, &[u32::MAX, u32::MAX]);

        let bytes = Std430Layout::finish(buf);

        #[rustfmt::skip]
        assert_eq!(
            &*bytes,
            &[
                // scalar
                u8::MAX, u8::MAX, u8::MAX, u8::MAX,
                // scalar
                u8::MAX, u8::MAX, u8::MAX, u8::MAX,
            ]
        );
    }

    #[test]
    fn struct_derive() {
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

        let a = a.as_gpu_bytes::<Std140Layout>();
        let a = Std140Layout::finish(a);

        assert_eq!(a.len(), 16);

        let b = b.as_gpu_bytes::<Std140Layout>();
        let b = Std140Layout::finish(b);

        assert_eq!(b.len(), 32);
    }
}

use std::borrow::Cow;

// re-export the derive crate
pub use gpu_layout_derive::AsGpuBytes;

pub trait GpuLayout {
    /// default implementation following standard layout rules
    fn write(buf: &mut GpuBytes, data: &impl AsGpuBytes) {
        let data = data.as_gpu_bytes::<Self>();

        // skip if empty
        if data.bytes.is_empty() {
            return;
        }

        // alignment is the max of all fields/elements
        buf.alignment = buf.alignment.max(data.alignment);

        // calculate and insert padding according to the alignment of the data we're inserting
        Self::align_to(buf, data.alignment);

        // insert the data itself
        buf.bytes.to_mut().extend_from_slice(&data.bytes);
    }

    fn write_array(buf: &mut GpuBytes, data: &[impl AsGpuBytes]);

    fn align_to(buf: &mut GpuBytes, alignment: usize) {
        let offset = buf.bytes.len();
        let padding = offset.next_multiple_of(alignment) - offset;

        buf.bytes.to_mut().extend(std::iter::repeat_n(0u8, padding));
    }

    /// makes sure the byte buffer as a whole is aligned and returns the raw data
    fn finish(mut buf: GpuBytes) -> Cow<'_, [u8]> {
        let alignment = buf.alignment;
        Self::align_to(&mut buf, alignment);

        buf.bytes
    }
}

pub struct Std140Layout;

impl GpuLayout for Std140Layout {
    fn write_array(buf: &mut GpuBytes, data: &[impl AsGpuBytes]) {
        for elem in data {
            let mut elem = elem.as_gpu_bytes::<Self>();

            // in std140 layout, array elements are aligned to 16 bytes
            let element_alignment = elem.alignment.next_multiple_of(16);
            Self::align_to(&mut elem, element_alignment);

            Self::write(buf, &elem);
        }
    }
}

pub struct Std430Layout;

impl GpuLayout for Std430Layout {
    fn write_array(buf: &mut GpuBytes, data: &[impl AsGpuBytes]) {
        for elem in data {
            let mut elem = elem.as_gpu_bytes::<Self>();

            // in std430 layout, array elements are aligned normally
            let element_alignment = elem.alignment;
            Self::align_to(&mut elem, element_alignment);

            Self::write(buf, &elem);
        }
    }
}

pub struct GpuBytes<'a> {
    bytes: Cow<'a, [u8]>,
    alignment: usize,
}

impl<'a> GpuBytes<'a> {
    pub fn empty() -> Self {
        Self {
            bytes: Vec::new().into(),
            alignment: 4,
        }
    }

    pub fn from_slice(data: &'a [u8], alignment: usize) -> Self {
        Self {
            bytes: Cow::Borrowed(data),
            alignment,
        }
    }

    pub fn finish(&mut self) -> Cow<'a, [u8]> {
        self.bytes.clone()
    }
}

pub trait AsGpuBytes {
    fn as_gpu_bytes<L: GpuLayout + ?Sized>(&self) -> GpuBytes;
}

impl AsGpuBytes for GpuBytes<'_> {
    fn as_gpu_bytes<L: GpuLayout + ?Sized>(&self) -> GpuBytes {
        GpuBytes {
            bytes: Cow::Borrowed(&self.bytes),
            alignment: self.alignment,
        }
    }
}

macro_rules! primitive_impl_gpu_bytes {
    ($datatype:ty, alignment = $alignment:literal) => {
        impl AsGpuBytes for $datatype {
            fn as_gpu_bytes<L: GpuLayout + ?Sized>(&self) -> GpuBytes {
                GpuBytes::from_slice(bytemuck::bytes_of(self), $alignment)
            }
        }
    };
    ($datatype:ty, columns = $columns:literal) => {
        impl AsGpuBytes for $datatype {
            fn as_gpu_bytes<L: GpuLayout + ?Sized>(&self) -> GpuBytes {
                let mut buf = GpuBytes::empty();

                for i in 0..$columns {
                    L::write(&mut buf, &self.col(i));
                }

                buf
            }
        }
    };
}

primitive_impl_gpu_bytes!(f32, alignment = 4);
primitive_impl_gpu_bytes!(glam::Vec2, alignment = 8);
primitive_impl_gpu_bytes!(glam::Vec3, alignment = 16);
primitive_impl_gpu_bytes!(glam::Vec4, alignment = 16);

primitive_impl_gpu_bytes!(i32, alignment = 4);
primitive_impl_gpu_bytes!(glam::IVec2, alignment = 8);
primitive_impl_gpu_bytes!(glam::IVec3, alignment = 16);
primitive_impl_gpu_bytes!(glam::IVec4, alignment = 16);

primitive_impl_gpu_bytes!(u32, alignment = 4);
primitive_impl_gpu_bytes!(glam::UVec2, alignment = 8);
primitive_impl_gpu_bytes!(glam::UVec3, alignment = 16);
primitive_impl_gpu_bytes!(glam::UVec4, alignment = 16);

primitive_impl_gpu_bytes!(glam::Mat2, columns = 2);
primitive_impl_gpu_bytes!(glam::Mat3, columns = 3);
primitive_impl_gpu_bytes!(glam::Mat4, columns = 4);

impl<T: AsGpuBytes> AsGpuBytes for &[T] {
    fn as_gpu_bytes<L: GpuLayout + ?Sized>(&self) -> GpuBytes {
        let mut buf = GpuBytes::empty();

        L::write_array(&mut buf, self);

        buf
    }
}

impl<T: AsGpuBytes, const N: usize> AsGpuBytes for [T; N] {
    fn as_gpu_bytes<L: GpuLayout + ?Sized>(&self) -> GpuBytes {
        let mut buf = GpuBytes::empty();

        L::write_array(&mut buf, self);

        buf
    }
}

impl<T: AsGpuBytes> AsGpuBytes for Vec<T> {
    fn as_gpu_bytes<L: GpuLayout + ?Sized>(&self) -> GpuBytes {
        let mut buf = GpuBytes::empty();

        L::write_array(&mut buf, self);

        buf
    }
}

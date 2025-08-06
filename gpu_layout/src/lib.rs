use std::{borrow::Cow, marker::PhantomData};

// re-export the derive crate
pub use gpu_layout_derive::AsGpuBytes;

pub trait GpuLayout {
    /// default implementation following standard layout rules
    fn write(buf: &mut GpuBytes<Self>, data: &impl AsGpuBytes) {
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

    fn write_array(buf: &mut GpuBytes<Self>, data: &[impl AsGpuBytes]);

    fn align_to(buf: &mut GpuBytes<Self>, alignment: usize) {
        let offset = buf.bytes.len();
        let padding = offset.next_multiple_of(alignment) - offset;

        // this `if` guard prevents copying/allocating new data if there is no padding required
        // as such, users can manually implement AsGpuBytes to completely avoid copying entirely
        // while still benefitting from this crate's api
        if padding != 0 {
            buf.bytes.to_mut().extend(std::iter::repeat_n(0u8, padding));
        }
    }

    /// aligns the buffer as a whole
    fn align(buf: &mut GpuBytes<Self>) {
        let alignment = buf.alignment;
        Self::align_to(buf, alignment);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Std140Layout;

impl GpuLayout for Std140Layout {
    fn write_array(buf: &mut GpuBytes<Self>, data: &[impl AsGpuBytes]) {
        for elem in data {
            let mut elem = elem.as_gpu_bytes::<Self>();

            // in std140 layout, array elements are aligned to 16 bytes
            let element_alignment = elem.alignment.next_multiple_of(16);
            Self::align_to(&mut elem, element_alignment);

            Self::write(buf, &elem);
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Std430Layout;

impl GpuLayout for Std430Layout {
    fn write_array(buf: &mut GpuBytes<Self>, data: &[impl AsGpuBytes]) {
        for elem in data {
            let mut elem = elem.as_gpu_bytes::<Self>();

            // in std430 layout, array elements are aligned normally
            let element_alignment = elem.alignment;
            Self::align_to(&mut elem, element_alignment);

            Self::write(buf, &elem);
        }
    }
}

#[derive(Debug, Clone)]
pub struct GpuBytes<'a, L: GpuLayout + ?Sized> {
    bytes: Cow<'a, [u8]>,
    alignment: usize,
    _layout: PhantomData<L>,
}

impl<'a, L: GpuLayout + ?Sized> GpuBytes<'a, L> {
    pub fn empty() -> Self {
        Self {
            bytes: Vec::new().into(),
            alignment: 4,
            _layout: PhantomData,
        }
    }

    pub fn from_slice(data: &'a [u8], alignment: usize) -> Self {
        Self {
            bytes: Cow::Borrowed(data),
            alignment,
            _layout: PhantomData,
        }
    }

    pub fn write(&mut self, data: &impl AsGpuBytes) -> &mut Self {
        L::write(self, data);
        self
    }

    /// Aligns the data according to the current alignment and returns it as a slice of bytes.
    pub fn as_slice(&mut self) -> &[u8] {
        L::align(self);
        &self.bytes
    }
}

pub trait AsGpuBytes {
    fn as_gpu_bytes<L: GpuLayout + ?Sized>(&self) -> GpuBytes<L>;
}

impl<L: GpuLayout + ?Sized> AsGpuBytes for GpuBytes<'_, L> {
    fn as_gpu_bytes<U: GpuLayout + ?Sized>(&self) -> GpuBytes<U> {
        GpuBytes {
            bytes: Cow::Borrowed(&self.bytes),
            alignment: self.alignment,
            _layout: PhantomData,
        }
    }
}

macro_rules! primitive_impl_gpu_bytes {
    ($datatype:ty, alignment = $alignment:literal) => {
        impl AsGpuBytes for $datatype {
            fn as_gpu_bytes<L: GpuLayout + ?Sized>(&self) -> GpuBytes<L> {
                GpuBytes::from_slice(bytemuck::bytes_of(self), $alignment)
            }
        }
    };
    ($datatype:ty, columns = $columns:literal) => {
        impl AsGpuBytes for $datatype {
            fn as_gpu_bytes<L: GpuLayout + ?Sized>(&self) -> GpuBytes<L> {
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
    fn as_gpu_bytes<L: GpuLayout + ?Sized>(&self) -> GpuBytes<L> {
        let mut buf = GpuBytes::empty();

        L::write_array(&mut buf, self);

        buf
    }
}

impl<T: AsGpuBytes, const N: usize> AsGpuBytes for [T; N] {
    fn as_gpu_bytes<L: GpuLayout + ?Sized>(&self) -> GpuBytes<L> {
        let mut buf = GpuBytes::empty();

        L::write_array(&mut buf, self);

        buf
    }
}

impl<T: AsGpuBytes> AsGpuBytes for Vec<T> {
    fn as_gpu_bytes<L: GpuLayout + ?Sized>(&self) -> GpuBytes<L> {
        let mut buf = GpuBytes::empty();

        L::write_array(&mut buf, self);

        buf
    }
}

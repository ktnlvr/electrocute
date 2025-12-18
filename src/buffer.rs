use std::{any::TypeId, marker::PhantomData};

use bytemuck::{Pod, Zeroable, bytes_of, from_bytes};

use crate::component::Component;

pub struct ComponentBuffer {
    buffer: Vec<u8>,
    stride: usize,
    type_id: TypeId,
}

#[derive(Debug, Clone, Copy, Zeroable)]
#[repr(C)]
struct ComponentStoredData<C: Component> {
    component: C,
    state: C::State,
}

unsafe impl<C: Component> Pod for ComponentStoredData<C> {}

impl ComponentBuffer {
    pub fn new<C: Component>() -> Self {
        let alignment = align_of::<ComponentStoredData<C>>();
        let size = size_of::<ComponentStoredData<C>>().max(1);

        let stride = ((size + alignment - 1) / alignment) * alignment;
        let type_id = TypeId::of::<C>();

        Self {
            buffer: vec![],
            stride,
            type_id,
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len() / self.stride
    }

    pub fn push<C: Component>(&mut self, component: C) {
        let data = ComponentStoredData {
            component,
            state: Default::default(),
        };

        let bytes = bytes_of(&data);
        assert!(bytes.len() <= self.stride);

        self.buffer.extend_from_slice(bytes);
        self.buffer
            .extend((0..(self.stride - bytes.len())).map(|_| 0));
    }

    pub fn iter<C: Component>(&self) -> impl Iterator<Item = (&C, &C::State)> {
        ComponentIterator {
            idx: 0,
            buffer: self,
            _phantom: PhantomData,
        }
    }

    pub fn iter_mut<C: Component>(&mut self) -> impl Iterator<Item = (&mut C, &mut C::State)> {
        ComponentIteratorMut {
            idx: 0,
            buffer: self,
            _phantom: PhantomData,
        }
    }
}

struct ComponentIterator<'buffer, C: Component> {
    idx: usize,
    buffer: &'buffer ComponentBuffer,
    _phantom: PhantomData<C>,
}

impl<'buffer, C: Component> Iterator for ComponentIterator<'buffer, C> {
    // XXX(Artur): Maybe use a sub-lifetime?
    type Item = (&'buffer C, &'buffer C::State);

    fn next(&mut self) -> Option<Self::Item> {
        if TypeId::of::<C>() != self.buffer.type_id {
            return None;
        }

        let count = self.buffer.buffer.len() / self.buffer.stride;
        if self.idx >= count {
            return None;
        }

        let start = self.idx * self.buffer.stride;
        let end = start + size_of::<C>();

        self.idx += 1;

        let slice = &self.buffer.buffer[start..end];
        let ComponentStoredData { component, state } = from_bytes(slice);

        Some((component, state))
    }
}
struct ComponentIteratorMut<'buffer, C: Component> {
    idx: usize,
    buffer: &'buffer mut ComponentBuffer,
    _phantom: PhantomData<C>,
}

impl<'buffer, C: Component> Iterator for ComponentIteratorMut<'buffer, C> {
    type Item = (&'buffer mut C, &'buffer mut C::State);

    fn next(&mut self) -> Option<Self::Item> {
        if TypeId::of::<C>() != self.buffer.type_id {
            return None;
        }

        let count = self.buffer.buffer.len() / self.buffer.stride;
        if self.idx >= count {
            return None;
        }

        let start = self.idx * self.buffer.stride;
        let end = start + size_of::<ComponentStoredData<C>>();

        self.idx += 1;

        let ptr = self.buffer.buffer[start..end].as_mut_ptr() as *mut ComponentStoredData<C>;
        let ComponentStoredData { component, state } = unsafe { &mut *ptr };

        Some((component, state))
    }
}

#[cfg(test)]
mod test {
    use bytemuck::{Pod, Zeroable};

    use crate::numerical::LinearEquations;

    use super::*;

    #[derive(Pod, Zeroable, Clone, Copy, Default)]
    #[repr(C)]
    pub struct DemoComponent {
        inner: u32,
    }

    impl Component for DemoComponent {
        type State = ();
        const TERMINAL_COUNT: usize = 0;
        const PRIORITY: usize = 0;

        fn stamp(
            &self,
            _net: &mut LinearEquations,
            _dt: f64,
            _terminals: [u32; Self::TERMINAL_COUNT],
            _state: &Self::State,
        ) {
            unreachable!()
        }
    }

    #[test]
    fn test_component_iteration() {
        let mut buffer = ComponentBuffer::new::<DemoComponent>();

        const N: u32 = 5;

        (0..N).for_each(|_| buffer.push(DemoComponent::default()));

        let n = buffer.iter::<DemoComponent>().fold(0u32, |acc, _| acc + 1);

        assert_eq!(n, N);
    }

    #[test]
    fn test_component_mut_iteration() {
        let mut buffer = ComponentBuffer::new::<DemoComponent>();

        const N: usize = 5;
        (0..N).for_each(|_| buffer.push(DemoComponent { inner: 0 }));

        for (comp, _) in buffer.iter_mut::<DemoComponent>() {
            *comp = DemoComponent { inner: 1 }
        }

        for (&DemoComponent { inner }, _) in buffer.iter::<DemoComponent>() {
            assert_eq!(inner, 1);
        }

        let n = buffer.iter::<DemoComponent>().count();
        assert_eq!(n, N);
    }
}

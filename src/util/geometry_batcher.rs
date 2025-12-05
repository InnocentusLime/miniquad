use std::rc::Rc;

use bytemuck::Pod;

use crate::{BufferUsage, GlContext, IndexBuffer, IndexBufferElement, VertexBuffer};

#[derive(Debug)]
pub struct GeometryBatcher<T: Pod + Default, I: IndexBufferElement = u16> {
    pub vertices: VertexBuffer<T>,
    pub indicies: IndexBuffer<I>,
    client_vertices: Vec<T>,
    client_indicies: Vec<I>,
}

impl<T: Pod + Default, I: IndexBufferElement> GeometryBatcher<T, I> {
    pub fn new_from_size(ctx: &Rc<GlContext>, vertices_size: usize, indicies_size: usize) -> Self {
        Self::new(
            ctx.new_empty_vertex_buffer(BufferUsage::Stream, vertices_size),
            ctx.new_empty_index_buffer(BufferUsage::Stream, indicies_size),
        )
    }

    pub fn new(vertices: VertexBuffer<T>, indicies: IndexBuffer<I>) -> Self {
        let client_vertices = Vec::with_capacity(vertices.size());
        let client_indicies = Vec::with_capacity(indicies.size());
        GeometryBatcher {
            vertices,
            indicies,
            client_vertices,
            client_indicies,
        }
    }

    pub fn extend(&mut self, vertices: &[T], indicies: &[I]) {
        let index_off = self.client_vertices.len();

        assert!(
            self.client_vertices.len() + vertices.len() <= self.vertices.size(),
            "vertex buffer will overfill"
        );
        assert!(
            self.client_indicies.len() + indicies.len() <= self.indicies.size(),
            "index buffer will overfill"
        );
        self.client_vertices.extend(vertices.iter().copied());
        self.client_indicies
            .extend(indicies.iter().copied().map(|x| x.offset_by(index_off)));
    }

    pub fn element_count(&self) -> u32 {
        self.client_indicies.len() as u32
    }

    pub fn finish(&mut self) -> u32 {
        let result = self.element_count();
        self.vertices.update(&self.client_vertices);
        self.indicies.update(&self.client_indicies);
        self.client_vertices.clear();
        self.client_indicies.clear();
        result
    }
}

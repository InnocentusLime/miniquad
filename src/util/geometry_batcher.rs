use std::rc::Rc;

use crate::graphics::{
    BufferUsage, GlContext, IndexBuffer, Result, Vertex, VertexBuffer, VertexIndex,
};

#[derive(Debug)]
pub struct GeometryBatcher<V: Vertex, I: VertexIndex = u16> {
    pub vertices: VertexBuffer<V>,
    pub indicies: IndexBuffer<I>,
    client_vertices: Vec<V>,
    client_indicies: Vec<I>,
}

impl<V: Vertex, I: VertexIndex> GeometryBatcher<V, I> {
    pub fn new_from_size(
        ctx: &Rc<GlContext>,
        vertices_size: usize,
        indicies_size: usize,
    ) -> Result<Self> {
        Ok(Self::new(
            ctx.new_empty_vertex_buffer(BufferUsage::Stream, vertices_size)?,
            ctx.new_empty_index_buffer(BufferUsage::Stream, indicies_size)?,
        ))
    }

    pub fn new(vertices: VertexBuffer<V>, indicies: IndexBuffer<I>) -> Self {
        let client_vertices = Vec::with_capacity(vertices.size());
        let client_indicies = Vec::with_capacity(indicies.size());
        GeometryBatcher { vertices, indicies, client_vertices, client_indicies }
    }

    #[track_caller]
    pub fn extend(&mut self, vertices: &[V], indicies: &[I]) {
        let index_off = self.client_vertices.len();

        debug_assert!(
            self.client_vertices.len() + vertices.len() <= self.vertices.size(),
            "vertex buffer will overfill"
        );
        debug_assert!(
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

    /// NOTE: flush does NOT clear the client-side buffers. You must do it yourself.
    pub fn flush(&mut self) -> u32 {
        let result = self.element_count();

        let vert_size = self.vertices.size().min(self.client_vertices.len());
        self.vertices.update(&self.client_vertices[0..vert_size]);

        let ind_size = self.indicies.size().min(self.client_indicies.len());
        self.indicies.update(&self.client_indicies[0..ind_size]);

        result
    }

    pub fn clear(&mut self) {
        self.client_vertices.clear();
        self.client_indicies.clear();
    }
}

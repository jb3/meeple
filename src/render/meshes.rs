use super::*;

impl Renderer {
    pub(super) fn upload(&self, m: &MeshData) -> Option<Mesh> {
        if m.idx.is_empty() {
            return None;
        }
        let gl = &self.gl;
        let vao = gl.create_vertex_array()?;
        gl.bind_vertex_array(Some(&vao));
        let vbo = gl.create_buffer()?;
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vbo));
        gl.buffer_data_with_u8_array(GL::ARRAY_BUFFER, m.bytes(), GL::STATIC_DRAW);
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_with_i32(0, 3, GL::FLOAT, false, 20, 0);
        gl.enable_vertex_attrib_array(1);
        gl.vertex_attrib_pointer_with_i32(1, 4, GL::UNSIGNED_BYTE, true, 20, 12);
        gl.enable_vertex_attrib_array(2);
        gl.vertex_attrib_pointer_with_i32(2, 4, GL::UNSIGNED_BYTE, false, 20, 16);
        let ibo = gl.create_buffer()?;
        gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&ibo));
        gl.buffer_data_with_u8_array(GL::ELEMENT_ARRAY_BUFFER, m.idx_bytes(), GL::STATIC_DRAW);
        gl.bind_vertex_array(None);
        Some(Mesh { vao, vbo, ibo, count: m.idx.len() as i32 })
    }

    pub fn add_chunk(&mut self, key: (i32, i32), solid: &MeshData, water: &MeshData) {
        self.remove_chunk(key);
        let m = ChunkMeshes { solid: self.upload(solid), water: self.upload(water) };
        self.meshes.insert(key, m);
    }

    pub fn remove_chunk(&mut self, key: (i32, i32)) {
        if let Some(m) = self.meshes.remove(&key) {
            for mesh in [m.solid, m.water].into_iter().flatten() {
                self.gl.delete_vertex_array(Some(&mesh.vao));
                self.gl.delete_buffer(Some(&mesh.vbo));
                self.gl.delete_buffer(Some(&mesh.ibo));
            }
        }
    }

    pub fn has_chunk(&self, key: (i32, i32)) -> bool {
        self.meshes.contains_key(&key)
    }
}

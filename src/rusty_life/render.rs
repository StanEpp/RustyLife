extern crate sdl2;
extern crate gl;
extern crate nalgebra as na;

use crate::rusty_life::input;
use crate::rusty_life::view;

use sdl2::*;

use std::ffi;
use std::ptr;

pub struct Renderer {
    sdl_context : Sdl,
    sdl_window : sdl2::video::Window,
    gl_context : sdl2::video::GLContext,
    shader_program : gl::types::GLuint,
    vao : gl::types::GLuint,
    vbo_cells : gl::types::GLuint,
    vbo_indices : gl::types::GLuint,
}

impl Drop for Renderer {
    fn drop(self : &mut Self) {
        unsafe {
            gl::DeleteBuffers(2,
                [self.vbo_indices, self.vbo_cells].as_ptr());
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}

impl Renderer {
    pub fn new(name : &str,
               window_size : (u32, u32),
               num_rows : u32, num_cols : u32) -> Renderer {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let sdl_window = video_subsystem.window(name, window_size.0, window_size.1)
                                        .position_centered()
                                        .opengl()
                                        .resizable()
                                        .build()
                                        .unwrap();

        let gl_context = sdl_window.gl_create_context().unwrap();
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

        video_subsystem.gl_set_swap_interval(sdl2::video::SwapInterval::Immediate).unwrap();

        let glattr = video_subsystem.gl_attr();
        glattr.set_context_version(4, 4);
        glattr.set_multisample_buffers(2);
        glattr.set_multisample_samples(8);

        let mut r = Renderer {
            sdl_context : sdl_context,
            sdl_window : sdl_window,
            gl_context : gl_context,
            shader_program : 0,
            vao : 0,
            vbo_cells : 0,
            vbo_indices : 0,
        };

        r.init_gl(num_rows, num_cols);

        r
    }

    fn init_gl(self : &mut Self,
               num_rows : u32, num_cols : u32) {
        unsafe {
            gl::Enable(gl::MULTISAMPLE);
            gl::Hint(gl::LINE_SMOOTH_HINT, gl::NICEST);
            gl::ClearColor(0.25, 0.25, 0.25, 1.0);

            // Create Shader
            let file_content = String::from(r"
            #version 430

            layout(location=0) in uint cellStrip;
            layout(location=1) in uvec2 coordinate;

            out uint gs_cellStrip;
            out uvec2 gs_coordinate;

            void main(void){
                gs_cellStrip = cellStrip;
                gs_coordinate = coordinate;
            }
            ");
            let shader_source = ffi::CString::new(file_content.as_bytes()).unwrap();
            let v_shader = gl::CreateShader(gl::VERTEX_SHADER);
            gl::ShaderSource(v_shader, 1, &shader_source.as_ptr(), ptr::null());
            gl::CompileShader(v_shader);
            self.check_compile_errors(v_shader, "VERTEX");


            let file_content = String::from(r"
            #version 430

            layout(points) in;
            layout(triangle_strip, max_vertices = 64) out;
            layout(location = 2) uniform mat4 MVP;

            in uint gs_cellStrip[]; // 16-bit
            in uvec2 gs_coordinate[];

            const float aspect_ratio = 16f / 9.f;


            void main(void){
                uint m = 1<<15;
                float cellLength = 1f;
                float cellGapLength = 0.1f;

                vec4 cell_offset = vec4(cellLength + cellGapLength, 0, 0, 0);

                float col = float(gs_coordinate[0].x) / 16.f;
                col = col * (cellLength + cellGapLength) * 16.f - 1.f;
                float row = 1 - float(gs_coordinate[0].y) * (cellLength + cellGapLength);

                vec4 cell_origin = vec4(col, row, 0, 1.f);

                for(int i = 0; i < 16; ++i) {
                    if(bool(gs_cellStrip[0] & m)) {
                        vec4 topLeft = cell_origin + cell_offset * float(i);
                        vec4 bottomLeft = topLeft +
                                          vec4(0, -cellLength, 0, 0);
                        vec4 topRight = topLeft +
                                          vec4(cellLength, 0, 0, 0);
                        vec4 bottomRight = bottomLeft +
                                          vec4(cellLength, 0, 0, 0);

                        gl_Position = MVP * topLeft;
                        EmitVertex();
                        gl_Position = MVP * bottomLeft;
                        EmitVertex();
                        gl_Position = MVP * topRight;
                        EmitVertex();
                        gl_Position = MVP * bottomRight;
                        EmitVertex();
                        EndPrimitive();
                    }
                    m = m >> 1;
                }
            }
            ");
            let shader_source = ffi::CString::new(file_content.as_bytes()).unwrap();
            let g_shader = gl::CreateShader(gl::GEOMETRY_SHADER);
            gl::ShaderSource(g_shader, 1, &shader_source.as_ptr(), ptr::null());
            gl::CompileShader(g_shader);
            self.check_compile_errors(g_shader, "GEOMETRY");


            let file_content = String::from(r"
            #version 430

            out vec4 color;

            void main(void){
                color = vec4(1.f);
            }
            ");
            let shader_source = ffi::CString::new(file_content.as_bytes()).unwrap();
            let f_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
            gl::ShaderSource(f_shader, 1, &shader_source.as_ptr(), ptr::null());
            gl::CompileShader(f_shader);
            self.check_compile_errors(f_shader, "FRAGMENT");


            let program = gl::CreateProgram();
            gl::AttachShader(program, v_shader);
            gl::AttachShader(program, g_shader);
            gl::AttachShader(program, f_shader);
            gl::LinkProgram(program);
            self.check_compile_errors(program, "PROGRAM");
            self.shader_program = program;

            let mut coordinates : Vec<u32> = Vec::new();
            for row in 0..num_rows {
                for col in 0..(num_cols / 16) { // sizeof(u16) == 16
                    coordinates.push(col * 16);
                    coordinates.push(row);
                }
            }

            let mut vbo = [0, 0];
            gl::GenBuffers(2, vbo.as_mut_ptr());
            self.vbo_cells = vbo[0];
            self.vbo_indices = vbo[1];

            gl::GenVertexArrays(1, &mut self.vao as *mut u32);
            gl::BindVertexArray(self.vao);

            // VBO for cell bits ; index -> 0
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo_cells);
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribIPointer(0, 1, gl::UNSIGNED_SHORT, 0, std::ptr::null());

            // VBO for coordinates ; index -> 1
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo_indices);
            gl::BufferStorage(gl::ARRAY_BUFFER,
                              (std::mem::size_of::<u32>() * coordinates.len()) as isize,
                              coordinates.as_ptr() as *const ffi::c_void,
                              0);
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribIPointer(1, 2, gl::UNSIGNED_INT, 0, std::ptr::null());

            gl::BindBuffer(0, gl::ARRAY_BUFFER);

            // Don't unbind the VAO because we only have one in the application
            // Bind shader program only once for the same reason
            gl::UseProgram(self.shader_program);
        }
    }

    pub fn render(self : &mut Self, cells : &Vec<u16>, view : &view::OrthoView, dt : &std::time::Duration) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // VAO already bound in init_gl()
            // Shader Program already bound in init_gl()

            // Buffer re-specification by orphaning the buffer before
            // filling it with new data
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo_cells);
            gl::BufferData(gl::ARRAY_BUFFER,
                           (cells.len() * std::mem::size_of::<u16>()) as isize,
                           std::ptr::null(),
                           gl::STREAM_DRAW);
            gl::BufferData(gl::ARRAY_BUFFER,
                           (cells.len() * std::mem::size_of::<u16>()) as isize,
                           cells.as_ptr() as *const ffi::c_void,
                           gl::STREAM_DRAW);
            gl::BindBuffer(0, gl::ARRAY_BUFFER);

            gl::UniformMatrix4fv(2, 1 , 0, view.mvp.as_ptr() as *const gl::types::GLfloat);

            gl::DrawArrays(gl::POINTS, 0, cells.len() as i32);
        }

        self.sdl_window.gl_swap_window();
    }

    pub fn create_input(self : &Self) -> input::Input {
        input::Input::new(&self.sdl_context)
    }

    unsafe fn check_compile_errors(&self, shader : u32, type_: &str) {
        let mut success = gl::FALSE as gl::types::GLint;
        let info_log = String::with_capacity(8192);
        let ptr = info_log.as_ptr();
        let returned_log_length : *mut i32 = &mut 0;
        if type_ != "PROGRAM" {
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
            if success != gl::TRUE as gl::types::GLint {
                gl::GetShaderInfoLog(
                    shader,
                    8192,
                    returned_log_length,
                    ptr as *mut gl::types::GLchar
                );
                std::mem::forget(info_log);
                let info_log_string = String::from_raw_parts(ptr as *mut u8, *returned_log_length as usize, 8192);
                println!(
                    "Failed to compile {} shader:\n{}",
                    type_, info_log_string
                );
            }
        } else {
            gl::GetProgramiv(shader, gl::LINK_STATUS, &mut success);
            if success != gl::TRUE as gl::types::GLint {
                gl::GetProgramInfoLog(
                    shader,
                    8192,
                    returned_log_length,
                    ptr as *mut gl::types::GLchar
                );
                std::mem::forget(info_log);
                let info_log_string = String::from_raw_parts(ptr as *mut u8, *returned_log_length as usize, 8192);
                println!(
                    "Failed to link {}:\n{} {}",
                    type_, info_log_string, *returned_log_length
                );
            }
        }
    }
}
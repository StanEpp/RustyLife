extern crate nalgebra as na;

use crate::rusty_life::input;

pub struct OrthoView {
    pub mvp : na::Matrix4<f32>,
    view_ortho_mat : na::Orthographic3<f32>,
}

impl OrthoView {
    pub fn new (window_size : (u32, u32)) -> OrthoView {
        let view_ortho = na::Orthographic3::new(-(window_size.0 as f32)/ 2., window_size.0 as f32/ 2.,
                                                -(window_size.1 as f32)/2., window_size.1 as f32 /2.,
                                                0., 1.);
        let mut r = OrthoView {
            mvp : view_ortho.as_matrix().clone(),
            view_ortho_mat : view_ortho,
        };

        r.reset_view();

        r
    }

    pub fn update(self : &mut Self, input_map : &input::InputMap, dt : &std::time::Duration) {
        let mut translation_speed = 2.; // pixels / second
        let mut scaling_speed = 1.05;
        let mut final_scaling = 1.;
        let mut final_translation = na::Vector3::new(0., 0., 0.);


        if input_map.keys_pressed[input::Key::LSHIFT] ||
           input_map.keys_hold[input::Key::LSHIFT] {
            translation_speed *= 3.;
            scaling_speed += 3. * (scaling_speed - 1.);
        }
        translation_speed *= dt.as_secs_f32();


        if input_map.keys_pressed[input::Key::W] ||
           input_map.keys_hold[input::Key::W] {
            final_translation.y -= translation_speed;
        }
        if input_map.keys_pressed[input::Key::A] ||
           input_map.keys_hold[input::Key::A] {
            final_translation.x += translation_speed;
        }
        if input_map.keys_pressed[input::Key::S] ||
           input_map.keys_hold[input::Key::S] {
            final_translation.y += translation_speed;
        }
        if input_map.keys_pressed[input::Key::D] ||
           input_map.keys_hold[input::Key::D] {
            final_translation.x -= translation_speed;
        }

        if input_map.keys_pressed[input::Key::MouseWheelUp] {
            final_scaling = scaling_speed;
        }
        if input_map.keys_pressed[input::Key::MouseWheelDown] {
            final_scaling = 1. / scaling_speed;
        }

        if input_map.keys_pressed[input::Key::MouseLeftButton] ||
           input_map.keys_hold[input::Key::MouseLeftButton] {
            final_translation.x += translation_speed * input_map.mouse_x_dt as f32;
            final_translation.y -= translation_speed * input_map.mouse_y_dt as f32;
        }

        if input_map.keys_pressed[input::Key::F] {
            self.reset_view();
        }


        self.mvp.append_nonuniform_scaling_mut(&na::Vector3::new(final_scaling, final_scaling, 1.));
        self.mvp.append_translation_mut(&final_translation);
    }

    fn reset_view(self : &mut Self) {
        self.mvp = self.view_ortho_mat.as_matrix().clone();
        self.mvp.append_translation_mut(&na::Vector3::new(-1., 1., 0.));
    }
}
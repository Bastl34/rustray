use nalgebra::{Vector3, Matrix4};

use crate::{helper, shape::ShapeBasics};

#[derive(Debug)]
pub struct Frame
{
    pub object_name: String,

    pub translation: Option<Vector3<f32>>,
    pub rotation: Option<Vector3<f32>>,
    pub scale: Option<Vector3<f32>>
}

impl Frame
{
    pub fn new(object_name: String, translation: Option<Vector3<f32>>, rotation: Option<Vector3<f32>>, scale: Option<Vector3<f32>>) -> Frame
    {
        Frame
        {
            object_name: object_name.clone(),

            translation: translation,
            rotation: rotation,
            scale: scale
        }
    }
}

#[derive(Debug)]
pub struct Keyframe
{
    pub time: u64,

    pub objects: Vec<Frame>
}

impl Keyframe
{
    pub fn new(time: u64, objects: Vec<Frame>) -> Keyframe
    {
        Keyframe
        {
            time: time,

            objects: objects,
        }
    }
}

#[derive(Debug)]
pub struct Animation
{
    pub enabled: bool,
    pub fps: u32,

    pub keyframes: Vec<Keyframe>
}

impl Animation
{
    pub fn new() -> Animation
    {
        Animation
        {
            enabled: false,
            fps: 25,

            keyframes: vec![],
        }
    }

    pub fn has_animation(&self) -> bool
    {
        self.enabled && self.get_frames_amount_to_render() > 0 && self.has_initial_keyframe() && self.keyframes.len() >= 2
    }

    pub fn has_initial_keyframe(&self) -> bool
    {
        if let Some(first) = self.keyframes.first()
        {
            return first.time == 0
        }

        false
    }

    pub fn get_frames_amount_to_render(&self) -> u64
    {
        let mut last_timestamp = 0;

        if let Some(last) = self.keyframes.last()
        {
            last_timestamp = last.time;
        }

        let last_frame = self.fps as f64 * (last_timestamp as f64 / 1000.0);

        last_frame.floor() as u64
    }

    pub fn get_keyframes_for_frame(&self, frame: u64) -> (&Keyframe, &Keyframe, f64)
    {
        let timestamp = ((1000.0 / self.fps as f64) * frame as f64).floor() as u64;

        let mut first_keyframe = self.keyframes.first().unwrap();
        let mut last_keyframe = self.keyframes.first().unwrap();

        for i in 0..self.keyframes.len()
        {
            if self.keyframes[i].time <= timestamp
            {
                first_keyframe = &self.keyframes[i];

                if i + 1 >= self.keyframes.len()
                {
                    last_keyframe = &self.keyframes[i];
                }
                else
                {
                    last_keyframe = &self.keyframes[i + 1];
                }
            }
        }

        let frame_time_pos = timestamp - first_keyframe.time;
        let keyframe_time_diff = last_keyframe.time - first_keyframe.time;
        let factor = (1.0 / keyframe_time_diff as f64) * frame_time_pos as f64;

        (first_keyframe, last_keyframe, factor)
    }

    pub fn get_trans_for_frame(&self, frame: u64, object_name: String) -> Option<Matrix4<f32>>
    {
        let (first, last, scale) = self.get_keyframes_for_frame(frame);

        let mut first_keyframe = None;
        let mut last_keyframe = None;

        for object in &first.objects
        {
            if object.object_name == object_name
            {
                first_keyframe = Some(object);
                break;
            }
        }

        for object in &last.objects
        {
            if object.object_name == object_name
            {
                last_keyframe = Some(object);
                break;
            }
        }

        if first_keyframe.is_none() || last_keyframe.is_none()
        {
            return None;
        }

        let first_keyframe = first_keyframe.unwrap();
        let last_keyframe = last_keyframe.unwrap();

        let scale_factor = scale as f32;

        //translation
        let mut translation = Vector3::<f32>::new(0.0, 0.0, 0.0);
        if let (Some(first_trans), Some(last_trans)) = (first_keyframe.translation, last_keyframe.translation)
        {
            translation.x = helper::interpolate(first_trans.x, last_trans.x, scale_factor);
            translation.y = helper::interpolate(first_trans.y, last_trans.y, scale_factor);
            translation.z = helper::interpolate(first_trans.z, last_trans.z, scale_factor);
        }

        //scale
        let mut scale = Vector3::<f32>::new(1.0, 1.0, 1.0);
        if let (Some(first_scale), Some(last_scale)) = (first_keyframe.scale, last_keyframe.scale)
        {
            scale.x = helper::interpolate(first_scale.x, last_scale.x, scale_factor);
            scale.y = helper::interpolate(first_scale.y, last_scale.y, scale_factor);
            scale.z = helper::interpolate(first_scale.z, last_scale.z, scale_factor);
        }

        //rotation
        let mut rotation = Vector3::<f32>::new(0.0, 0.0, 0.0);
        if let (Some(first_rot), Some(last_rot)) = (first_keyframe.rotation, last_keyframe.rotation)
        {
            rotation.x = helper::interpolate(first_rot.x, last_rot.x, scale_factor);
            rotation.y = helper::interpolate(first_rot.y, last_rot.y, scale_factor);
            rotation.z = helper::interpolate(first_rot.z, last_rot.z, scale_factor);
        }

        let trans = ShapeBasics::get_transformation(&Matrix4::<f32>::identity(), translation, scale, rotation);

        Some(trans)
    }

}

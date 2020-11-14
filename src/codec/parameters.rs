use std::{
    mem,
    ops::{Deref, DerefMut},
    rc::Rc,
    slice,
};

use crate::{format, Error};

use super::{Context, Id, Profile};
use ffi::*;
use media;
use ChannelLayout;

pub struct Parameters {
    ptr: *mut AVCodecParameters,
    owner: Option<Rc<dyn Drop>>,
}

unsafe impl Send for Parameters {}

impl Parameters {
    pub unsafe fn wrap(ptr: *mut AVCodecParameters, owner: Option<Rc<dyn Drop>>) -> Self {
        Parameters { ptr, owner }
    }

    pub unsafe fn as_ptr(&self) -> *const AVCodecParameters {
        self.ptr as *const _
    }

    pub unsafe fn as_mut_ptr(&mut self) -> *mut AVCodecParameters {
        self.ptr
    }
}

impl Parameters {
    pub fn new() -> Self {
        unsafe {
            Parameters {
                ptr: avcodec_parameters_alloc(),
                owner: None,
            }
        }
    }

    pub fn medium(&self) -> media::Type {
        unsafe { media::Type::from((*self.as_ptr()).codec_type) }
    }

    pub fn id(&self) -> Id {
        unsafe { Id::from((*self.as_ptr()).codec_id) }
    }

    #[inline]
    pub fn extradata(&self) -> Option<&[u8]> {
        unsafe {
            if (*self.as_ptr()).extradata.is_null() {
                None
            } else {
                Some(slice::from_raw_parts(
                    (*self.as_ptr()).extradata,
                    (*self.as_ptr()).extradata_size as usize,
                ))
            }
        }
    }

    pub fn bit_rate(&self) -> usize {
        unsafe { (*self.as_ptr()).bit_rate as usize }
    }

    pub fn profile(&self) -> Profile {
        unsafe { Profile::from((self.id(), (*self.as_ptr()).profile)) }
    }

    pub fn level(&self) -> i32 {
        unsafe { (*self.as_ptr()).level as i32 }
    }

    pub fn video(mut self) -> Result<Video, Error> {
        match self.medium() {
            media::Type::Unknown => {
                unsafe {
                    (*self.as_mut_ptr()).codec_type = media::Type::Video.into();
                }

                Ok(Video(self))
            }

            media::Type::Video => Ok(Video(self)),

            _ => Err(Error::InvalidData),
        }
    }

    pub fn audio(mut self) -> Result<Audio, Error> {
        match self.medium() {
            media::Type::Unknown => {
                unsafe {
                    (*self.as_mut_ptr()).codec_type = media::Type::Audio.into();
                }

                Ok(Audio(self))
            }

            media::Type::Audio => Ok(Audio(self)),

            _ => Err(Error::InvalidData),
        }
    }
}

impl Default for Parameters {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Parameters {
    fn drop(&mut self) {
        unsafe {
            if self.owner.is_none() {
                avcodec_parameters_free(&mut self.as_mut_ptr());
            }
        }
    }
}

impl Clone for Parameters {
    fn clone(&self) -> Self {
        let mut ctx = Parameters::new();
        ctx.clone_from(self);

        ctx
    }

    fn clone_from(&mut self, source: &Self) {
        unsafe {
            avcodec_parameters_copy(self.as_mut_ptr(), source.as_ptr());
        }
    }
}

impl<C: AsRef<Context>> From<C> for Parameters {
    fn from(context: C) -> Parameters {
        let mut parameters = Parameters::new();
        let context = context.as_ref();
        unsafe {
            avcodec_parameters_from_context(parameters.as_mut_ptr(), context.as_ptr());
        }
        parameters
    }
}

pub struct Video(pub Parameters);

impl Video {
    pub fn format(&self) -> format::Pixel {
        unsafe {
            format::Pixel::from(mem::transmute::<_, AVPixelFormat>(
                (*self.0.as_ptr()).format,
            ))
        }
    }

    pub fn width(&self) -> u32 {
        unsafe { (*self.0.as_ptr()).width as u32 }
    }

    pub fn height(&self) -> u32 {
        unsafe { (*self.0.as_ptr()).height as u32 }
    }
}

impl Deref for Video {
    type Target = Parameters;

    #[inline(always)]
    fn deref(&self) -> &<Self as Deref>::Target {
        &self.0
    }
}

impl DerefMut for Video {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut <Self as Deref>::Target {
        &mut self.0
    }
}

pub struct Audio(pub Parameters);

impl Audio {
    pub fn format(&self) -> format::Sample {
        unsafe {
            format::Sample::from(mem::transmute::<_, AVSampleFormat>(
                (*self.0.as_ptr()).format,
            ))
        }
    }

    pub fn rate(&self) -> u32 {
        unsafe { (*self.0.as_ptr()).sample_rate as u32 }
    }

    pub fn channel_layout(&self) -> ChannelLayout {
        unsafe { ChannelLayout::from_bits_truncate((*self.0.as_ptr()).channel_layout) }
    }

    pub fn channels(&self) -> u16 {
        unsafe { (*self.0.as_ptr()).channels as u16 }
    }
}

impl Deref for Audio {
    type Target = Parameters;

    #[inline(always)]
    fn deref(&self) -> &<Self as Deref>::Target {
        &self.0
    }
}

impl DerefMut for Audio {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut <Self as Deref>::Target {
        &mut self.0
    }
}

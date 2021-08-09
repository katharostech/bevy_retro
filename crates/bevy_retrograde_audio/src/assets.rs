use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use kira::sound::{handle::SoundHandle as KiraSoundHandle, Sound as KiraSound};

pub(crate) fn add_assets(app: &mut App) {
    app.add_asset::<SoundData>()
        .add_asset_loader(SoundDataLoader);
}

/// An asset that holds the data necessary to create a sound using the [`SoundController`][`crate::SoundController`] resource
///
/// Users will most-likely not interact with this type directly but can pass it to
/// [`create_sound`][`crate::SoundController::create_sound`].
#[derive(Clone, Debug, TypeUuid)]
#[uuid = "0b6b6127-a10a-4c67-938f-76f079a6f631"]
pub enum SoundData {
    Sound(KiraSound),
    SoundHandle(KiraSoundHandle),
}

/// An error that occurs when loading a sound asset
#[derive(thiserror::Error, Debug)]
pub enum SoundDataLoaderError {
    #[error("Non-unicode filenames are not supported")]
    NonUnicodeFilename,
    #[error("Error loading sound from file: {0}")]
    FileError(#[from] kira::sound::error::SoundFromFileError),
}

/// An LDTK map asset loader
#[derive(Default)]
struct SoundDataLoader;

impl AssetLoader for SoundDataLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        // Create a future for the load function
        Box::pin(async move { Ok(load_sound(bytes, load_context).await?) })
    }

    fn extensions(&self) -> &[&str] {
        &[
            #[cfg(feature = "mp3")]
            "mp3",
            #[cfg(feature = "ogg")]
            "ogg",
            #[cfg(feature = "flac")]
            "flac",
            #[cfg(feature = "wav")]
            "wav",
        ]
    }
}

async fn load_sound<'a, 'b>(
    bytes: &'a [u8],
    load_context: &'a mut LoadContext<'b>,
) -> Result<(), SoundDataLoaderError> {
    let sound = match load_context.path().extension() {
        Some(ext) => match ext
            .to_str()
            .ok_or(SoundDataLoaderError::NonUnicodeFilename)?
        {
            #[cfg(feature = "mp3")]
            "mp3" => KiraSound::from_mp3_reader(bytes, Default::default()),
            #[cfg(feature = "flac")]
            "flac" => KiraSound::from_flac_reader(bytes, Default::default()),
            #[cfg(feature = "ogg")]
            "ogg" => {
                let reader = std::io::Cursor::new(bytes);
                KiraSound::from_ogg_reader(reader, Default::default())
            }
            #[cfg(feature = "wav")]
            "wav" => KiraSound::from_wav_reader(bytes, Default::default()),
            _ => panic!("Unsupported sound extension, bevy should have caught this"),
        },
        None => {
            panic!("File does not have extension, bevy should have caught this")
        }
    }?;

    load_context.set_default_asset(LoadedAsset::new(SoundData::Sound(sound)));

    Ok(())
}

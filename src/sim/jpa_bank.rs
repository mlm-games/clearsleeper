//! The JPA resource bank: `JPAResourceManager` / `JPAEmitterResource` (`JPAResourceManager.cpp`)
//! over the emitter archive loader (`JPAEmitterLoader.cpp`).
//!
//! One `JPAC` archive is one resource manager. `JPAEmitterArchiveLoaderDataBase::load`
//! (`JPAEmitterLoader.cpp:73-94`) checks the magic and the version and hands the file to
//! `JPAEmitterArchiveLoader_v10::load` (`:97-186`), which walks it in one pass:
//!
//! * a `JPAEmitterArchiveData_v10` header (`:24-29`): the `JPAC` magic, the version at `0x04`,
//!   the emitter count at `0x08` and the texture count at `0x0A`; the walk starts at `0x20`
//!   (`:102`).
//! * per emitter, a `JPAEmitterParticleHeader_v10` (`:31-41`): the `JEFF` magic, the type at
//!   `0x04`, the block count at `0x0C`, the key, field and texture counts at `0x14`, `0x15` and
//!   `0x16`, and the resource id at `0x18`; the blocks begin at `+0x20` (`:131`).
//! * per block, a `JPAEmitterBlockHeader_v10` (`:43-48`): a four-byte tag, the size at `0x04`,
//!   and a walk that advances by that size (`:171`).
//! * after the emitters, `texResNum` `JUTDataBlockHeader` textures (`:178-185`,
//!   `JUTDataHeader.h:8-11`), whose size at `0x04` also advances the walk (`:184`).
//!
//! `JPAEmitterResource::registration` stores each emitter in archive order and writes the
//! particle header's `resID` into `JPAEmitterData::userIndex` (`JPAEmitterLoader.cpp:174`,
//! `JPAResourceManager.cpp:51-58`). `getByUserIndex` and `checkUserIndexDuplication` scan that
//! array in registration order (`JPAResourceManager.cpp:60-76`), so a bank answers the way the
//! loader filled it. [`JpaBank`] keys one such manager per `JPAEmitterManager::pResMgrArray` slot
//! (`JPAEmitterManager.h:46`).
//!
//! What the bank keeps is the table the reference reads: the resource ids, the three per-emitter
//! counts and each block's tag, size and offset. The block bodies are not decoded - the
//! `JPADynamicsBlockArc` family is a second parse this module does not own - so a resource here
//! answers "does this manager hold this user index, and where" and nothing else. In particular
//! the archive carries no group id: the group and the resource-manager selector are the caller's,
//! taken from the packed effect id (`d_particle.cpp:698-701`, `:725-730`).

/// `pResMgrArray[8]` (`JPAEmitterManager.h:46`).
pub const MANAGER_COUNT: usize = 8;

/// The `JPAC` archive magic `JPAEmitterArchiveLoaderDataBase::load` tests (`JPAEmitterLoader.cpp:77`).
pub const ARCHIVE_MAGIC: u32 = u32::from_be_bytes(*b"JPAC");
/// The `1-00` version it tests behind the magic (`JPAEmitterLoader.cpp:79`).
pub const ARCHIVE_VERSION: u32 = u32::from_be_bytes(*b"1-00");
/// The particle header's `JEFF` magic (`JPAEmitterLoader.cpp:32`).
pub const PARTICLE_MAGIC: u32 = u32::from_be_bytes(*b"JEFF");
/// The particle header's `jpa1` type (`JPAEmitterLoader.cpp:33`), carried but never read by the
/// reference.
pub const PARTICLE_TYPE: u32 = u32::from_be_bytes(*b"jpa1");

/// Where the first particle header begins (`JPAEmitterLoader.cpp:102`).
pub const ARCHIVE_HEADER_SIZE: usize = 0x20;
/// How far the blocks sit behind a particle header (`JPAEmitterLoader.cpp:131`).
pub const PARTICLE_HEADER_SIZE: usize = 0x20;
/// How big a block header is before its payload (`JPAEmitterLoader.cpp:43-48`).
pub const BLOCK_HEADER_SIZE: usize = 0x0C;
/// How big a texture entry's header is (`JUTDataHeader.h:8-11`).
pub const TEXTURE_HEADER_SIZE: usize = 8;

/// The eight block tags `JPAEmitterArchiveLoader_v10::load` switches on (`JPAEmitterLoader.cpp:135-169`).
pub const BLOCK_DYNAMICS: u32 = u32::from_be_bytes(*b"BEM1");
pub const BLOCK_FIELD: u32 = u32::from_be_bytes(*b"FLD1");
pub const BLOCK_KEY: u32 = u32::from_be_bytes(*b"KFA1");
pub const BLOCK_BASE_SHAPE: u32 = u32::from_be_bytes(*b"BSP1");
pub const BLOCK_EXTRA_SHAPE: u32 = u32::from_be_bytes(*b"ESP1");
pub const BLOCK_SWEEP_SHAPE: u32 = u32::from_be_bytes(*b"SSP1");
pub const BLOCK_EX_TEX: u32 = u32::from_be_bytes(*b"ETX1");
pub const BLOCK_TEX_DATA: u32 = u32::from_be_bytes(*b"TDB1");
/// The texture entry tag (`JPAEmitterLoader.cpp:179-184` reads a `JUTDataBlockHeader` there).
pub const TEXTURE_MAGIC: u32 = u32::from_be_bytes(*b"TEX1");

/// One `JPAEmitterBlockHeader_v10` (`JPAEmitterLoader.cpp:43-48`): its tag, the size that advances
/// the walk (`:171`) and where the header sits in the archive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JpaBlock {
    pub magic: u32,
    pub size: u32,
    pub offset: usize,
}

/// One registered emitter resource: the parsed form of a `JPAEmitterParticleHeader_v10` as
/// `JPAEmitterData` holds it after `JPAEmitterResource::registration` (`JPAResourceManager.cpp:51-58`).
///
/// Retail's `JPAEmitterData` also owns the decoded block objects; this carries the table the
/// loader read, which is what the registration and the lookup scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JpaResource {
    /// `JPAEmitterData::userIndex` (`JPAEmitterLoader.h:26`), written from `ptcl->resID`
    /// (`JPAEmitterLoader.cpp:174`).
    pub user_index: u16,
    /// `JPAEmitterParticleHeader_v10::keyNum` (`JPAEmitterLoader.cpp:37`, copied at `:117`).
    pub key_num: u8,
    /// `JPAEmitterParticleHeader_v10::fldNum` (`JPAEmitterLoader.cpp:38`, copied at `:121`).
    pub fld_num: u8,
    /// `JPAEmitterParticleHeader_v10::textureNum` (`JPAEmitterLoader.cpp:39`, copied at `:125`).
    pub texture_num: u8,
    /// The blocks the header's `blockNum` names (`JPAEmitterLoader.cpp:132-172`), in file order.
    pub blocks: Vec<JpaBlock>,
}

/// A resolved bank resource: which manager slot holds it, the registration index
/// `getByUserIndex` found it at, and the user index itself. This is the port's counterpart of the
/// `JPADataBlockLinkInfo` retail pins on an emitter (`JPAEmitter.cpp:134`, `mpDataLinkInfo =
/// info`), narrowed to where the resource lives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JpaResourceRef {
    pub rm_id: u8,
    pub index: usize,
    pub user_index: u16,
}

/// One `JUTDataBlockHeader` texture (`JPAEmitterLoader.cpp:178-185`): its tag, the size that
/// advances the walk and its offset. `JPATextureArc` decodes the image itself; the bank carries
/// the entry the walk passed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JpaTexture {
    pub magic: u32,
    pub size: u32,
    pub offset: usize,
}

/// `JPAResourceManager` (`JPAResourceManager.h:45-56`) reduced to the two resource lists its
/// loader fills: `pEmtrRes`'s registered emitters and `pTexRes`'s textures. The heap and the
/// decoded texture objects are the renderer's.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct JpaResourceManager {
    resources: Vec<JpaResource>,
    textures: Vec<JpaTexture>,
}

impl JpaResourceManager {
    /// `JPAResourceManager::getEmitterResource()->getByUserIndex(userIndex)`
    /// (`JPAResourceManager.cpp:61-67`): the first registered resource whose user index matches,
    /// in archive order.
    pub fn get_by_user_index(&self, user_index: u16) -> Option<&JpaResource> {
        self.find_by_user_index(user_index)
            .map(|(_, resource)| resource)
    }

    /// [`JpaResourceManager::get_by_user_index`] together with the registration index the
    /// resource was found at - the position `JPAEmitterResource::registration` gave it
    /// (`JPAResourceManager.cpp:51-58`).
    pub fn find_by_user_index(&self, user_index: u16) -> Option<(usize, &JpaResource)> {
        let index = self.registration_index(user_index)?;
        Some((index, &self.resources[index]))
    }

    /// The registration index `getByUserIndex` found the resource at.
    pub fn registration_index(&self, user_index: u16) -> Option<usize> {
        self.resources
            .iter()
            .position(|resource| resource.user_index == user_index)
    }

    /// `JPAEmitterResource::checkUserIndexDuplication` (`JPAResourceManager.cpp:70-76`).
    pub fn check_user_index_duplication(&self, user_index: u16) -> bool {
        self.get_by_user_index(user_index).is_some()
    }

    pub fn resources(&self) -> &[JpaResource] {
        &self.resources
    }

    pub fn textures(&self) -> &[JpaTexture] {
        &self.textures
    }

    pub fn is_empty(&self) -> bool {
        self.resources.is_empty() && self.textures.is_empty()
    }

    /// `JPAEmitterArchiveLoaderDataBase::load` (`JPAEmitterLoader.cpp:73-94`) over
    /// `JPAEmitterArchiveLoader_v10::load` (`:97-186`). The reference trusts every count and
    /// every size past the two header checks, so a walk that would leave the buffer faults there;
    /// the port answers with a [`JpaBankError`] instead.
    fn parse(data: &[u8]) -> Result<Self, JpaBankError> {
        let magic = read_u32(data, 0)?;
        if magic != ARCHIVE_MAGIC {
            return Err(JpaBankError::BadArchiveMagic { magic });
        }
        let version = read_u32(data, 4)?;
        if version != ARCHIVE_VERSION {
            return Err(JpaBankError::BadArchiveVersion { version });
        }
        let emitter_count = read_u16(data, 8)? as usize;
        let texture_count = read_u16(data, 0x0A)? as usize;

        let mut resources = Vec::new();
        let mut offset = ARCHIVE_HEADER_SIZE;
        for _ in 0..emitter_count {
            let particle_magic = read_u32(data, offset)?;
            if particle_magic != PARTICLE_MAGIC {
                return Err(JpaBankError::BadParticleMagic {
                    at: offset,
                    magic: particle_magic,
                });
            }
            let block_count = read_u32(data, offset + 0x0C)?;
            let key_num = read_u8(data, offset + 0x14)?;
            let fld_num = read_u8(data, offset + 0x15)?;
            let texture_num = read_u8(data, offset + 0x16)?;
            let user_index = read_u16(data, offset + 0x18)?;

            let mut blocks = Vec::new();
            let mut block_offset = offset + PARTICLE_HEADER_SIZE;
            for _ in 0..block_count {
                let block_magic = read_u32(data, block_offset)?;
                let size = read_u32(data, block_offset + 4)?;
                if size == 0 {
                    return Err(JpaBankError::ZeroSize { at: block_offset });
                }
                let end = entry_end(block_offset, size, data.len())?;
                blocks.push(JpaBlock {
                    magic: block_magic,
                    size,
                    offset: block_offset,
                });
                block_offset = end;
            }
            resources.push(JpaResource {
                user_index,
                key_num,
                fld_num,
                texture_num,
                blocks,
            });
            offset = block_offset;
        }

        let mut textures = Vec::new();
        for _ in 0..texture_count {
            let texture_magic = read_u32(data, offset)?;
            let size = read_u32(data, offset + 4)?;
            if size == 0 {
                return Err(JpaBankError::ZeroSize { at: offset });
            }
            let end = entry_end(offset, size, data.len())?;
            textures.push(JpaTexture {
                magic: texture_magic,
                size,
                offset,
            });
            offset = end;
        }

        Ok(JpaResourceManager {
            resources,
            textures,
        })
    }
}

/// `JPAEmitterManager::pResMgrArray[8]` (`JPAEmitterManager.h:46`): the resource managers the
/// emitter-creation path resolves ids through. A slot is empty until an archive is mounted into
/// it and empty again after `clearResourceManager` (`JPAEmitterManager.cpp:139-150`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JpaBank {
    managers: [Option<JpaResourceManager>; MANAGER_COUNT],
}

impl Default for JpaBank {
    /// `JPAEmitterManager::JPAEmitterManager`'s `pResMgrArray[i] = 0` loop
    /// (`JPAEmitterManager.cpp:24-25`).
    fn default() -> Self {
        JpaBank {
            managers: core::array::from_fn(|_| None),
        }
    }
}

impl JpaBank {
    /// `pResMgrArray[rmID]`. `None` is the reference's NULL slot.
    pub fn manager(&self, rm_id: u8) -> Option<&JpaResourceManager> {
        self.managers.get(usize::from(rm_id))?.as_ref()
    }

    /// `pResMgrArray[rmID]->getEmitterResource()->getByUserIndex(user_index)`.
    pub fn get_by_user_index(&self, rm_id: u8, user_index: u16) -> Option<&JpaResource> {
        self.manager(rm_id)?.get_by_user_index(user_index)
    }

    /// The registration index of `get_by_user_index`'s answer.
    pub fn registration_index(&self, rm_id: u8, user_index: u16) -> Option<usize> {
        self.manager(rm_id)?.registration_index(user_index)
    }

    /// `pResMgrArray[rmID]->getEmitterResource()->checkUserIndexDuplication(user_index)`.
    pub fn check_user_index_duplication(&self, rm_id: u8, user_index: u16) -> bool {
        self.manager(rm_id)
            .is_some_and(|manager| manager.check_user_index_duplication(user_index))
    }

    /// `JPAResourceManager::JPAResourceManager(data, heap)` (`JPAResourceManager.cpp:79-82`) over
    /// `JPAEmitterArchiveLoaderDataBase::load`: parse `data` and put the result in the slot
    /// `rm_id` names, replacing whatever was mounted there - the same assignment
    /// `dPa_control_c::createRoomScene` makes to `pResMgrArray[1]` (`d_particle.cpp:744`).
    ///
    /// The eight-slot bound is the reference's own `rmID < 8` (`JPAEmitterManager.cpp:54`,
    /// `:140`); the slot index is checked here instead of asserted.
    pub fn register(&mut self, rm_id: u8, data: &[u8]) -> Result<(), JpaBankError> {
        let slot = self
            .managers
            .get_mut(usize::from(rm_id))
            .ok_or(JpaBankError::ManagerOutOfRange { rm_id })?;
        let manager = JpaResourceManager::parse(data)?;
        *slot = Some(manager);
        Ok(())
    }

    /// `JPAEmitterManager::clearResourceManager`'s `pResMgrArray[rmID] = NULL`
    /// (`JPAEmitterManager.cpp:150`). The emitter sweep that precedes it (`:141-149`) is
    /// [`super::jpa::Jpa::clear_resource_manager`]'s: the emitters live in the pool beside this
    /// bank, not in it, so the two halves of the reference's one call sit on either side.
    pub fn clear(&mut self, rm_id: u8) {
        if let Some(slot) = self.managers.get_mut(usize::from(rm_id)) {
            *slot = None;
        }
    }
}

/// Why an archive could not become a resource manager. Retail warns and leaves both resource
/// pointers unset on a bad magic or version (`JPAEmitterLoader.cpp:77-93`) and trusts the counts
/// after that; a walk that would leave the buffer faults there, so the port reports it here at
/// load rather than panicking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JpaBankError {
    /// `rm_id` is not one of `pResMgrArray`'s eight slots. Retail asserts `rmID < 8`
    /// (`JPAEmitterManager.cpp:54`, `:140`).
    ManagerOutOfRange { rm_id: u8 },
    /// The archive magic is not `JPAC` (`JPAEmitterLoader.cpp:77`, warn 202).
    BadArchiveMagic { magic: u32 },
    /// The archive version is not `1-00` (`JPAEmitterLoader.cpp:79-90`, warn 201).
    BadArchiveVersion { version: u32 },
    /// The read at `at` needs `need` bytes and the archive holds `len`. The reference reads on
    /// into whatever follows the buffer.
    Truncated { at: usize, need: usize, len: usize },
    /// A particle header's magic is not `JEFF` (`JPAEmitterLoader.cpp:32`). The reference never
    /// tests it, so this is the port's own check on a walk that has lost its place.
    BadParticleMagic { at: usize, magic: u32 },
    /// A block or texture size is zero, which would leave the walk stationary
    /// (`JPAEmitterLoader.cpp:171`, `:184`).
    ZeroSize { at: usize },
    /// A block or texture entry runs past the archive end (`JPAEmitterLoader.cpp:171`, `:184`).
    OutOfRange { at: usize, size: u32, len: usize },
}

impl std::fmt::Display for JpaBankError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ManagerOutOfRange { rm_id } => {
                write!(
                    formatter,
                    "resource manager {rm_id} is outside pResMgrArray"
                )
            }
            Self::BadArchiveMagic { magic } => {
                write!(formatter, "archive magic {magic:#010x} is not JPAC")
            }
            Self::BadArchiveVersion { version } => {
                write!(formatter, "archive version {version:#010x} is not 1-00")
            }
            Self::Truncated { at, need, len } => {
                write!(
                    formatter,
                    "read of {need} bytes at {at} past the {len}-byte archive"
                )
            }
            Self::BadParticleMagic { at, magic } => {
                write!(
                    formatter,
                    "particle header at {at} is {magic:#010x}, not JEFF"
                )
            }
            Self::ZeroSize { at } => write!(formatter, "entry at {at} has size zero"),
            Self::OutOfRange { at, size, len } => {
                write!(
                    formatter,
                    "entry at {at} of {size} bytes past the {len}-byte archive"
                )
            }
        }
    }
}

fn read_u32(data: &[u8], at: usize) -> Result<u32, JpaBankError> {
    let bytes = data.get(at..at + 4).ok_or(JpaBankError::Truncated {
        at,
        need: 4,
        len: data.len(),
    })?;
    Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn read_u16(data: &[u8], at: usize) -> Result<u16, JpaBankError> {
    let bytes = data.get(at..at + 2).ok_or(JpaBankError::Truncated {
        at,
        need: 2,
        len: data.len(),
    })?;
    Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
}

fn read_u8(data: &[u8], at: usize) -> Result<u8, JpaBankError> {
    data.get(at).copied().ok_or(JpaBankError::Truncated {
        at,
        need: 1,
        len: data.len(),
    })
}

/// The end of an entry whose header sits at `at` and whose size field is `size`
/// (`JPAEmitterLoader.cpp:171`, `:184`), refused when it would leave the archive.
fn entry_end(at: usize, size: u32, len: usize) -> Result<usize, JpaBankError> {
    at.checked_add(size as usize)
        .filter(|end| *end <= len)
        .ok_or(JpaBankError::OutOfRange { at, size, len })
}

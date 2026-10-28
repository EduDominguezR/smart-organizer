//! Movimiento seguro de archivos.
//!
//! Nunca sobrescribe: si el destino existe, anade (1), (2). Entre discos copia,
//! verifica tamano y hash, y solo entonces borra el original. Si la verificacion
//! falla, el original queda intacto y la copia parcial se elimina.

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::error::Error;

/// Una ruta que Windows pueda abrir aunque pase de 260 caracteres.
pub fn op_path(path: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        let text = path.as_os_str().to_string_lossy();
        if text.len() > 240 && !text.starts_with(r"\\?\") && path.is_absolute() {
            return PathBuf::from(format!(r"\\?\{}", text.trim_start_matches(r"\\?\")));
        }
    }

    path.to_path_buf()
}

pub fn is_locked(path: &Path) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        // share_mode(0) pide acceso exclusivo: si otro proceso lo tiene abierto, falla.
        fs::OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(op_path(path))
            .is_err()
    }

    #[cfg(not(windows))]
    {
        // En Linux y macOS no existe un equivalente fiable; la estabilidad de
        // tamano que aplica el vigilante es el unico criterio.
        let _ = path;
        false
    }
}

pub fn unique_path(dir: &Path, name: &str) -> PathBuf {
    let candidate = dir.join(name);
    if !candidate.exists() {
        return candidate;
    }

    let path = Path::new(name);
    let stem = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_else(|| name.to_string());
    let ext = path
        .extension()
        .map(|ext| format!(".{}", ext.to_string_lossy()))
        .unwrap_or_default();

    for index in 1..10_000u32 {
        let next = dir.join(format!("{stem} ({index}){ext}"));
        if !next.exists() {
            return next;
        }
    }

    dir.join(format!("{stem} (copia){ext}"))
}

pub fn perform_move(from: &Path, dest_dir: &Path, name: &str) -> Result<PathBuf, Error> {
    if !from.exists() {
        return Err(Error::NotFound {
            path: from.to_path_buf(),
        });
    }

    fs::create_dir_all(op_path(dest_dir)).map_err(|err| Error::io(dest_dir, err))?;
    let dest = unique_path(dest_dir, name);

    match fs::rename(op_path(from), op_path(&dest)) {
        Ok(()) => Ok(dest),
        Err(err) if is_cross_device(&err) => {
            copy_verify_then_remove(from, &dest)?;
            Ok(dest)
        }
        Err(err) => Err(Error::io(from, err)),
    }
}

pub fn perform_copy(from: &Path, dest_dir: &Path, name: &str) -> Result<PathBuf, Error> {
    if !from.exists() {
        return Err(Error::NotFound {
            path: from.to_path_buf(),
        });
    }

    fs::create_dir_all(op_path(dest_dir)).map_err(|err| Error::io(dest_dir, err))?;
    let dest = unique_path(dest_dir, name);
    copy_verify(from, &dest)?;
    Ok(dest)
}

fn is_cross_device(err: &std::io::Error) -> bool {
    #[cfg(windows)]
    {
        // ERROR_NOT_SAME_DEVICE
        err.raw_os_error() == Some(17)
    }
    #[cfg(unix)]
    {
        // EXDEV
        err.raw_os_error() == Some(18)
    }
    #[cfg(not(any(windows, unix)))]
    {
        let _ = err;
        false
    }
}

fn copy_verify_then_remove(from: &Path, dest: &Path) -> Result<(), Error> {
    copy_verify(from, dest)?;
    fs::remove_file(op_path(from)).map_err(|err| Error::io(from, err))
}

fn copy_verify(from: &Path, dest: &Path) -> Result<(), Error> {
    let partial = dest.with_file_name(format!(
        "{}.sopart",
        dest.file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default()
    ));

    let result = (|| -> Result<(), Error> {
        let mut source = fs::File::open(op_path(from)).map_err(|err| Error::io(from, err))?;
        let mut target =
            fs::File::create(op_path(&partial)).map_err(|err| Error::io(&partial, err))?;

        let mut buffer = vec![0u8; 512 * 1024];
        loop {
            let read = source
                .read(&mut buffer)
                .map_err(|err| Error::io(from, err))?;
            if read == 0 {
                break;
            }
            target
                .write_all(&buffer[..read])
                .map_err(|err| Error::io(&partial, err))?;
        }
        target.flush().map_err(|err| Error::io(&partial, err))?;
        target.sync_all().map_err(|err| Error::io(&partial, err))?;
        drop(target);
        drop(source);

        let source_size = fs::metadata(op_path(from))
            .map_err(|err| Error::io(from, err))?
            .len();
        let partial_size = fs::metadata(op_path(&partial))
            .map_err(|err| Error::io(&partial, err))?
            .len();

        if source_size != partial_size {
            return Err(Error::CopyVerify {
                path: from.to_path_buf(),
            });
        }

        if hash_file(from)? != hash_file(&partial)? {
            return Err(Error::CopyVerify {
                path: from.to_path_buf(),
            });
        }

        fs::rename(op_path(&partial), op_path(dest)).map_err(|err| Error::io(dest, err))?;
        Ok(())
    })();

    if result.is_err() {
        let _ = fs::remove_file(op_path(&partial));
    }

    result
}

pub fn hash_file(path: &Path) -> Result<blake3::Hash, Error> {
    let mut file = fs::File::open(op_path(path)).map_err(|err| Error::io(path, err))?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = vec![0u8; 256 * 1024];

    loop {
        let read = file.read(&mut buffer).map_err(|err| Error::io(path, err))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(hasher.finalize())
}

pub fn send_to_trash(path: &Path) -> Result<(), Error> {
    trash::delete(path).map_err(|err| Error::trash(path, err.to_string()))
}

/// Mueve el contenido de una carpeta a otra sin sobrescribir nada. Si la carpeta
/// destino no existe se renombra entera; si existe, se mezcla archivo a archivo.
pub fn move_folder(old: &Path, new: &Path) -> Result<usize, Error> {
    if !old.exists() || old == new {
        return Ok(0);
    }

    // En Windows los nombres solo distinguen mayusculas en pantalla. Tratar eso
    // como una mezcla renombraria cada archivo a "nombre (1)".
    if cfg!(windows)
        && old
            .to_string_lossy()
            .eq_ignore_ascii_case(&new.to_string_lossy())
    {
        let _ = fs::rename(op_path(old), op_path(new));
        return Ok(0);
    }

    if !new.exists() {
        if let Some(parent) = new.parent() {
            fs::create_dir_all(op_path(parent)).map_err(|err| Error::io(parent, err))?;
        }
        match fs::rename(op_path(old), op_path(new)) {
            Ok(()) => return Ok(0),
            Err(err) if is_cross_device(&err) => {}
            Err(err) => return Err(Error::io(old, err)),
        }
    }

    merge_folder(old, new)
}

fn merge_folder(from: &Path, to: &Path) -> Result<usize, Error> {
    fs::create_dir_all(op_path(to)).map_err(|err| Error::io(to, err))?;
    let entries = fs::read_dir(op_path(from)).map_err(|err| Error::io(from, err))?;
    let mut moved = 0;

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(_) => continue,
        };

        if file_type.is_dir() {
            moved += merge_folder(&path, &to.join(&name))?;
        } else if file_type.is_file() {
            perform_move(&path, to, &name)?;
            moved += 1;
        }
    }

    let _ = fs::remove_dir(op_path(from));
    Ok(moved)
}

/// Devuelve un archivo a su sitio durante un deshacer. No renombra: si el
/// destino original esta ocupado, lo deja estar y lo informa.
pub fn restore(current: &Path, original: &Path) -> Result<(), Error> {
    if !current.exists() {
        return Err(Error::NotFound {
            path: current.to_path_buf(),
        });
    }

    if original.exists() {
        return Err(Error::Io {
            path: original.to_path_buf(),
            source: std::io::Error::new(std::io::ErrorKind::AlreadyExists, "destino ocupado"),
        });
    }

    if let Some(parent) = original.parent() {
        fs::create_dir_all(op_path(parent)).map_err(|err| Error::io(parent, err))?;
    }

    match fs::rename(op_path(current), op_path(original)) {
        Ok(()) => Ok(()),
        Err(err) if is_cross_device(&err) => {
            copy_verify_then_remove(current, original)?;
            Ok(())
        }
        Err(err) => Err(Error::io(current, err)),
    }
}

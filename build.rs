use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");

    #[cfg(windows)]
    {
        use winres::VersionInfo;

        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
        let icon = Path::new(&manifest_dir).join("assets").join("icon.ico");
        let out = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
        let rc = out.join("resource.rc");
        let obj = out.join("resource.o");

        let mut resource = winres::WindowsResource::new();
        resource
            .set_icon(icon.to_str().expect("ruta de icono"))
            .set_language(0x0409)
            .set("ProductName", "Smart Organizer")
            .set("FileDescription", env!("CARGO_PKG_DESCRIPTION"))
            .set("FileVersion", env!("CARGO_PKG_VERSION"))
            .set("ProductVersion", env!("CARGO_PKG_VERSION"))
            .set("LegalCopyright", "MIT")
            .set_version_info(VersionInfo::FILETYPE, 1)
            .set_manifest(
                r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <assemblyIdentity version="1.0.0.0" processorArchitecture="*" name="SmartOrganizer" type="win32"/>
  <description>Smart Organizer</description>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
  <application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
      <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true/pm</dpiAware>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">permonitorv2</dpiAwareness>
      <longPathAware xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">true</longPathAware>
    </windowsSettings>
  </application>
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"/>
    </application>
  </compatibility>
</assembly>"#,
            );

        resource
            .write_resource_file(&rc)
            .expect("no se pudo escribir resource.rc");

        // winres 0.1.12 pasa -I<ruta> sin comillas y esta carpeta tiene
        // espacios. Se compila el .rc ya generado con Command, que si cita
        // cada argumento, y se enlaza el objeto a mano.
        let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
        if target_env == "gnu" {
            let status = Command::new("windres.exe")
                .arg("-i")
                .arg(&rc)
                .arg("-o")
                .arg(&obj)
                .status()
                .expect("no se pudo ejecutar windres");
            assert!(status.success(), "windres fallo al compilar el icono");

            // MinGW anade su propio default-manifest.o desde el spec *endfile.
            // El nuestro ya trae DPI y longPathAware, asi que se quita solo esa
            // entrada y se compila el ejecutable con el spec resultante.
            let specs = Command::new("gcc.exe")
                .arg("-dumpspecs")
                .output()
                .expect("no se pudo leer los specs de gcc");
            assert!(specs.status.success(), "gcc -dumpspecs fallo");
            let text = String::from_utf8_lossy(&specs.stdout);
            let marker = "*endfile:";
            let start = text
                .find(marker)
                .map(|index| index + marker.len())
                .expect("gcc no tiene seccion endfile");
            let rest = &text[start..];
            let end = rest.find("\n*").unwrap_or(rest.len());
            let block = rest[..end].replace("%{!shared:%:if-exists(default-manifest.o%s)}", "");
            let spec_path = out.join("no-default-manifest.specs");
            std::fs::write(&spec_path, format!("*endfile:{block}"))
                .expect("no se pudo escribir el spec de enlace");
            println!("cargo:rustc-link-arg-bins=-specs={}", spec_path.display());

            // Se pasa el objeto directamente: dentro de un .a no tiene
            // simbolos que referenciar y el enlazador GNU no lo extraeria.
            println!("cargo:rustc-link-arg-bins={}", obj.display());
        } else {
            // MSVC: rc.exe recibe las rutas con /I y /fo, que Command cita.
            let status = Command::new("rc.exe")
                .arg(format!("/I{}", manifest_dir))
                .arg(format!("/fo{}", out.join("resource.lib").display()))
                .arg(&rc)
                .status()
                .expect("no se pudo ejecutar rc.exe");
            assert!(status.success(), "rc.exe fallo al compilar el icono");
            println!(
                "cargo:rustc-link-arg-bins={}",
                out.join("resource.lib").display()
            );
        }
    }
}

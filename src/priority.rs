//! Prioridad baja para el trabajo que no debe competir con la ventana.
//!
//! Windows planifica antes lo que el usuario esta mirando. El vigilante y los
//! escaneos pesados se quedan por debajo de lo normal; no se usa tiempo real ni
//! modo de fondo agresivo, que retrasaria las copias del propio usuario.

#[cfg(windows)]
mod win {
    use std::ffi::c_void;

    const BELOW_NORMAL_PRIORITY_CLASS: u32 = 0x0000_4000;
    const THREAD_PRIORITY_BELOW_NORMAL: i32 = -1;

    // PROCESS_INFORMATION_CLASS::ProcessPowerThrottling y
    // POWER_THROTTLING_CURRENT_VERSION.
    const PROCESS_POWER_THROTTLING: i32 = 4;
    const POWER_THROTTLING_VERSION: u32 = 1;
    const POWER_THROTTLING_EXECUTION_SPEED: u32 = 0x1;

    #[repr(C)]
    struct PowerThrottlingState {
        version: u32,
        control_mask: u32,
        state_mask: u32,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetCurrentProcess() -> *mut c_void;
        fn GetCurrentThread() -> *mut c_void;
        fn SetPriorityClass(process: *mut c_void, class: u32) -> i32;
        fn SetThreadPriority(thread: *mut c_void, priority: i32) -> i32;
        fn SetProcessInformation(
            process: *mut c_void,
            class: i32,
            information: *mut c_void,
            size: u32,
        ) -> i32;
    }

    pub fn lower_process() {
        // Si el sistema no lo permite, el proceso sigue funcionando igual.
        unsafe {
            SetPriorityClass(GetCurrentProcess(), BELOW_NORMAL_PRIORITY_CLASS);
        }
    }

    pub fn lower_thread() {
        unsafe {
            SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_BELOW_NORMAL);
        }
    }

    /// EcoQoS: el vigilante acepta velocidad de ejecucion reducida cuando no
    /// hay trabajo urgente. Si el sistema no lo soporta, no pasa nada.
    pub fn eco_qos() {
        let mut state = PowerThrottlingState {
            version: POWER_THROTTLING_VERSION,
            control_mask: POWER_THROTTLING_EXECUTION_SPEED,
            state_mask: POWER_THROTTLING_EXECUTION_SPEED,
        };
        unsafe {
            SetProcessInformation(
                GetCurrentProcess(),
                PROCESS_POWER_THROTTLING,
                (&mut state as *mut PowerThrottlingState).cast::<c_void>(),
                std::mem::size_of::<PowerThrottlingState>() as u32,
            );
        }
    }
}

pub fn lower_process() {
    #[cfg(windows)]
    win::lower_process();
}

pub fn lower_thread() {
    #[cfg(windows)]
    win::lower_thread();
}

/// Solo para el proceso `--watch`. La ventana nunca lo llama.
pub fn eco_qos() {
    #[cfg(windows)]
    win::eco_qos();
}

#![deny(unsafe_code)]

use std::collections::BTreeMap;

use zellij_resource_status::{format_resource_output, Config, CpuSample, MemorySample};
use zellij_tile::prelude::*;

const CPU_CONTEXT: &str = "cpu";
const VM_STAT_CONTEXT: &str = "vm_stat";
const MEM_TOTAL_CONTEXT: &str = "mem_total";
const PUBLISH_CONTEXT: &str = "publish";

#[derive(Default)]
struct ResourceStatusPlugin {
    config: Config,
    latest_cpu: Option<CpuSample>,
    latest_memory: Option<MemorySample>,
    pending_vm_stat: Option<String>,
    pending_total_bytes: Option<u64>,
    permissions_granted: bool,
}

impl ZellijPlugin for ResourceStatusPlugin {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        self.config = Config::from_zellij_config(&configuration);
        set_selectable(false);
        hide_self();
        request_permission(&[
            PermissionType::RunCommands,
            PermissionType::MessageAndLaunchOtherPlugins,
        ]);
        subscribe(&[
            EventType::PermissionRequestResult,
            EventType::RunCommandResult,
            EventType::Timer,
        ]);
        set_timeout(0.1);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::PermissionRequestResult(PermissionStatus::Granted) => {
                self.permissions_granted = true;
                self.sample_resources();
                true
            }
            Event::PermissionRequestResult(PermissionStatus::Denied) => true,
            Event::Timer(_) => {
                if self.permissions_granted {
                    self.sample_resources();
                }
                set_timeout(self.config.interval_secs());
                true
            }
            Event::RunCommandResult(exit_code, stdout, _stderr, context) => {
                if exit_code == Some(0) {
                    self.handle_command_result(&stdout, &context);
                }
                true
            }
            _ => false,
        }
    }
}

impl ResourceStatusPlugin {
    fn sample_resources(&mut self) {
        run_command(
            &["/usr/sbin/iostat", "-C", "-w", "1", "-c", "2"],
            request_context(CPU_CONTEXT),
        );
        run_command(&["/usr/bin/vm_stat"], request_context(VM_STAT_CONTEXT));
        run_command(
            &["/usr/sbin/sysctl", "-n", "hw.memsize"],
            request_context(MEM_TOTAL_CONTEXT),
        );
    }

    fn handle_command_result(&mut self, stdout: &[u8], context: &BTreeMap<String, String>) {
        let Some(request) = context.get("request") else {
            return;
        };
        if request == PUBLISH_CONTEXT {
            return;
        }

        let output = String::from_utf8_lossy(stdout);
        match request.as_str() {
            CPU_CONTEXT => {
                if let Some(cpu) = CpuSample::from_iostat(&output) {
                    self.latest_cpu = Some(cpu);
                }
            }
            VM_STAT_CONTEXT => {
                self.pending_vm_stat = Some(output.into_owned());
                self.refresh_memory_sample();
            }
            MEM_TOTAL_CONTEXT => {
                self.pending_total_bytes = output.trim().parse::<u64>().ok();
                self.refresh_memory_sample();
            }
            _ => return,
        }

        self.publish_if_ready();
    }

    fn refresh_memory_sample(&mut self) {
        let (Some(vm_stat), Some(total_bytes)) =
            (self.pending_vm_stat.as_deref(), self.pending_total_bytes)
        else {
            return;
        };

        if let Some(memory) = MemorySample::from_vm_stat_and_total(vm_stat, total_bytes) {
            self.latest_memory = Some(memory);
        }
    }

    fn publish_if_ready(&self) {
        let (Some(cpu), Some(memory)) = (self.latest_cpu, self.latest_memory) else {
            return;
        };

        let output = format_resource_output(cpu, memory, &self.config);
        let payload = self.config.pipe_payload(&output);
        let mut message = MessageToPlugin::new("zellij-resource-status").with_payload(payload);
        if let Some(url) = self.config.zjstatus_plugin_url() {
            message = message.with_plugin_url(url);
        }
        pipe_message_to_plugin(message);
    }
}

fn request_context(request: &str) -> BTreeMap<String, String> {
    BTreeMap::from([("request".to_string(), request.to_string())])
}

register_plugin!(ResourceStatusPlugin);

import { open } from "@tauri-apps/plugin-dialog";

export async function openExecutableFileDialog(): Promise<string | null> {
  const selected = await open({
    multiple: false,
    directory: false,
    title: "Выберите исполняемый файл",
  });

  return typeof selected === "string" ? selected : null;
}


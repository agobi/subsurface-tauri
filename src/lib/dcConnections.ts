// AI-generated (Claude)
import { load } from "@tauri-apps/plugin-store";

export type Transport = "Serial" | "BLE" | "Bluetooth" | "USBHID";

export interface DcConnectionEntry {
  lastTransport: Transport;
  addresses: Partial<Record<Transport, string>>;
}

export type DcConnections = Record<string, DcConnectionEntry>;

export async function loadDcConnections(): Promise<DcConnections> {
  const store = await load("settings.json");
  const saved = await store.get<DcConnections>("dcConnections");
  return saved ?? {};
}

export async function saveDcConnection(deviceKey: string, transport: Transport, address: string): Promise<void> {
  const store = await load("settings.json");
  const all = (await store.get<DcConnections>("dcConnections")) ?? {};
  const existing = all[deviceKey] ?? { lastTransport: transport, addresses: {} };
  const addresses = { ...existing.addresses };
  if (address) addresses[transport] = address;
  all[deviceKey] = { lastTransport: transport, addresses };
  await store.set("dcConnections", all);
  await store.save();
}

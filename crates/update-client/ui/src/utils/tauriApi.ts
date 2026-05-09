import { invoke } from '@tauri-apps/api/core';
import {
  RegisteredServer,
  SignedManifest,
  VerificationResult,
  AvailableUpdate,
  IntegrityReport,
  SecurityInfo,
} from '../types';

export const api = {
  // Server management
  getServers: (): Promise<RegisteredServer[]> =>
    invoke('get_servers'),

  addServer: (url: string): Promise<RegisteredServer> =>
    invoke('add_server', { url }),

  removeServer: (publisherId: string): Promise<boolean> =>
    invoke('remove_server', { publisherId }),

  // Updates
  checkUpdates: (publisherId: string, packageName: string): Promise<SignedManifest | null> =>
    invoke('check_updates', { publisherId, packageName }),

  checkAllUpdates: (): Promise<AvailableUpdate[]> =>
    invoke('check_all_updates'),

  verifyManifest: (manifest: SignedManifest): Promise<VerificationResult> =>
    invoke('verify_manifest', { manifest }),

  // Security
  getIntegrityReport: (): Promise<IntegrityReport> =>
    invoke('get_integrity_report'),

  getSecurityInfo: (): Promise<SecurityInfo> =>
    invoke('get_security_info'),
};
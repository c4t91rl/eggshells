import { create } from 'zustand';
import {
  RegisteredServer,
  AvailableUpdate,
  IntegrityReport,
  SecurityInfo,
} from '../types';

type Page = 'dashboard' | 'servers' | 'updates' | 'security' | 'settings' | 'logs';

interface AppStore {
  // Navigation
  currentPage: Page;
  setPage: (page: Page) => void;

  // Servers
  servers: RegisteredServer[];
  setServers: (servers: RegisteredServer[]) => void;

  // Updates
  availableUpdates: AvailableUpdate[];
  setAvailableUpdates: (updates: AvailableUpdate[]) => void;
  isCheckingUpdates: boolean;
  setCheckingUpdates: (checking: boolean) => void;

  // Security
  integrityReport: IntegrityReport | null;
  setIntegrityReport: (report: IntegrityReport) => void;
  securityInfo: SecurityInfo | null;
  setSecurityInfo: (info: SecurityInfo) => void;

  // Logs
  logs: LogEntry[];
  addLog: (level: LogLevel, message: string) => void;
  clearLogs: () => void;
}

export type LogLevel = 'info' | 'warn' | 'error' | 'success';

export interface LogEntry {
  id: number;
  timestamp: Date;
  level: LogLevel;
  message: string;
}

let logCounter = 0;

export const useAppStore = create<AppStore>((set) => ({
  currentPage: 'dashboard',
  setPage: (page) => set({ currentPage: page }),

  servers: [],
  setServers: (servers) => set({ servers }),

  availableUpdates: [],
  setAvailableUpdates: (updates) => set({ availableUpdates: updates }),
  isCheckingUpdates: false,
  setCheckingUpdates: (checking) => set({ isCheckingUpdates: checking }),

  integrityReport: null,
  setIntegrityReport: (report) => set({ integrityReport: report }),
  securityInfo: null,
  setSecurityInfo: (info) => set({ securityInfo: info }),

  logs: [],
  addLog: (level, message) =>
    set((state) => ({
      logs: [
        ...state.logs,
        { id: ++logCounter, timestamp: new Date(), level, message },
      ].slice(-500),
    })),
  clearLogs: () => set({ logs: [] }),
}));
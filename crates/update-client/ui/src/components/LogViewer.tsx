import React, { useRef, useEffect } from 'react';
import { useAppStore, LogEntry, LogLevel } from '../store/appStore';
import { HiOutlineTrash } from 'react-icons/hi2';

const LogViewer: React.FC = () => {
  const { logs, clearLogs } = useAppStore();
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [logs]);

  const levelConfig: Record<LogLevel, { color: string; label: string; icon: string }> = {
    info: { color: 'text-blue-400', label: 'INFO', icon: 'ℹ️' },
    warn: { color: 'text-yellow-400', label: 'WARN', icon: '⚠️' },
    error: { color: 'text-red-400', label: 'ERROR', icon: '❌' },
    success: { color: 'text-green-400', label: 'OK', icon: '✅' },
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-dark-50">Activity Log</h2>
          <p className="text-dark-400 text-sm mt-1">
            {logs.length} entries • Real-time system activity
          </p>
        </div>
        <button onClick={clearLogs} className="btn-danger flex items-center gap-2">
          <HiOutlineTrash size={18} />
          Clear Logs
        </button>
      </div>

      <div className="glass-card p-4 max-h-[70vh] overflow-y-auto font-mono text-sm">
        {logs.length === 0 ? (
          <div className="text-center text-dark-500 py-12">
            No log entries yet
          </div>
        ) : (
          <div className="space-y-1">
            {logs.map((log) => {
              const config = levelConfig[log.level];
              return (
                <div
                  key={log.id}
                  className="flex items-start gap-3 py-1.5 px-2 rounded hover:bg-dark-800/30"
                >
                  <span className="text-dark-600 text-xs whitespace-nowrap">
                    {log.timestamp.toLocaleTimeString()}
                  </span>
                  <span className={`${config.color} text-xs font-bold w-12`}>
                    {config.icon} {config.label}
                  </span>
                  <span className="text-dark-300 text-xs flex-1">{log.message}</span>
                </div>
              );
            })}
            <div ref={bottomRef} />
          </div>
        )}
      </div>
    </div>
  );
};

export default LogViewer;
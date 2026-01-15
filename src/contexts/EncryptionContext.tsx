import { createContext, useContext, useState, ReactNode } from 'react';

export interface LogEntry {
  id: string;
  timestamp: Date;
  message: string;
  type: 'info' | 'success' | 'error' | 'warning' | 'progress';
  details?: string;
}

interface EncryptionContextType {
  logs: LogEntry[];
  isEncrypting: boolean;
  addLog: (message: string, type?: LogEntry['type'], details?: string) => void;
  clearLogs: () => void;
  setIsEncrypting: (value: boolean) => void;
}

const EncryptionContext = createContext<EncryptionContextType | undefined>(undefined);

export function EncryptionProvider({ children }: { children: ReactNode }) {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [isEncrypting, setIsEncrypting] = useState(false);

  const addLog = (message: string, type: LogEntry['type'] = 'info', details?: string) => {
    const entry: LogEntry = {
      id: `${Date.now()}-${Math.random()}`,
      timestamp: new Date(),
      message,
      type,
      details,
    };
    setLogs((prev) => [...prev, entry]);
  };

  const clearLogs = () => {
    setLogs([]);
  };

  return (
    <EncryptionContext.Provider value={{ logs, isEncrypting, addLog, clearLogs, setIsEncrypting }}>
      {children}
    </EncryptionContext.Provider>
  );
}

export function useEncryption() {
  const context = useContext(EncryptionContext);
  if (context === undefined) {
    throw new Error('useEncryption must be used within an EncryptionProvider');
  }
  return context;
}

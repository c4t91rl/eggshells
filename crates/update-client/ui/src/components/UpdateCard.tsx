import React, { useState } from 'react';
import { AvailableUpdate } from '../types';
import VerificationBadge from './VerificationBadge';
import {
  HiOutlineChevronDown,
  HiOutlineChevronUp,
  HiOutlineDocumentText,
  HiOutlineFingerPrint,
  HiOutlineClock,
} from 'react-icons/hi2';
import { motion, AnimatePresence } from 'framer-motion';

interface Props {
  update: AvailableUpdate;
}

const UpdateCard: React.FC<Props> = ({ update }) => {
  const [expanded, setExpanded] = useState(false);
  const { manifest, verification, publisher_name } = update;

  const formatBytes = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / 1048576).toFixed(1)} MB`;
  };

  const totalSize = manifest.manifest.files.reduce((sum, f) => sum + f.size, 0);

  return (
    <div className="glass-card overflow-hidden">
      {/* Header */}
      <div
        className="p-5 cursor-pointer hover:bg-dark-800/30 transition-colors"
        onClick={() => setExpanded(!expanded)}
      >
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <div
              className={`w-12 h-12 rounded-xl flex items-center justify-center text-lg font-bold ${
                verification.is_valid
                  ? 'bg-green-500/15 text-green-400'
                  : 'bg-red-500/15 text-red-400'
              }`}
            >
              {manifest.manifest.package_name.charAt(0).toUpperCase()}
            </div>
            <div>
              <h4 className="font-semibold text-dark-100 text-lg">
                {manifest.manifest.package_name}
              </h4>
              <div className="flex items-center gap-3 text-sm text-dark-400">
                <span>v{manifest.manifest.version}</span>
                <span>•</span>
                <span>{publisher_name}</span>
                <span>•</span>
                <span>{formatBytes(totalSize)}</span>
              </div>
            </div>
          </div>

          <div className="flex items-center gap-3">
            {manifest.signatures.map((sig, idx) => (
              <VerificationBadge key={idx} algorithm={sig.algorithm} />
            ))}

            {verification.is_valid ? (
              <span className="status-ok">✓ Verified</span>
            ) : (
              <span className="status-error">✗ Failed</span>
            )}

            <button className="btn-primary text-sm py-1.5 px-3">
              Install
            </button>

            {expanded ? (
              <HiOutlineChevronUp size={20} className="text-dark-400" />
            ) : (
              <HiOutlineChevronDown size={20} className="text-dark-400" />
            )}
          </div>
        </div>
      </div>

      {/* Expanded Details */}
      <AnimatePresence>
        {expanded && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: 'auto', opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            className="overflow-hidden"
          >
            <div className="px-5 pb-5 border-t border-dark-700/50 pt-4">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                {/* Verification Details */}
                <div>
                  <h5 className="text-sm font-semibold text-dark-300 mb-3 flex items-center gap-2">
                    <HiOutlineFingerPrint size={16} />
                    Verification Checks
                  </h5>
                  <div className="space-y-2">
                    {verification.checks.map((check, idx) => (
                      <div
                        key={idx}
                        className="flex items-start gap-2 text-sm bg-dark-800/50 p-2.5 rounded-lg"
                      >
                        <span className={check.passed ? 'text-green-400' : 'text-red-400'}>
                          {check.passed ? '✓' : '✗'}
                        </span>
                        <div className="flex-1">
                          <div className="text-dark-200 font-medium">{check.name}</div>
                          <div className="text-dark-500 text-xs mt-0.5">{check.details}</div>
                        </div>
                      </div>
                    ))}
                  </div>

                  {verification.warnings.length > 0 && (
                    <div className="mt-3">
                      <h6 className="text-xs font-semibold text-yellow-400 mb-1">Warnings</h6>
                      {verification.warnings.map((w, idx) => (
                        <p key={idx} className="text-xs text-yellow-400/70">{w}</p>
                      ))}
                    </div>
                  )}
                </div>

                {/* File Details */}
                <div>
                  <h5 className="text-sm font-semibold text-dark-300 mb-3 flex items-center gap-2">
                    <HiOutlineDocumentText size={16} />
                    Files
                  </h5>
                  <div className="space-y-2">
                    {manifest.manifest.files.map((file, idx) => (
                      <div
                        key={idx}
                        className="bg-dark-800/50 p-3 rounded-lg"
                      >
                        <div className="flex justify-between text-sm">
                          <span className="text-dark-200 font-mono text-xs">{file.path}</span>
                          <span className="text-dark-400 text-xs">{formatBytes(file.size)}</span>
                        </div>
                        <div className="mt-1">
                          <code className="text-xs text-dark-500 font-mono break-all">
                            {file.hash_algorithm}: {file.hash.substring(0, 32)}...
                          </code>
                        </div>
                      </div>
                    ))}
                  </div>

                  {/* Release Notes */}
                  {manifest.manifest.release_notes && (
                    <div className="mt-4">
                      <h5 className="text-sm font-semibold text-dark-300 mb-2">Release Notes</h5>
                      <p className="text-sm text-dark-400 bg-dark-800/50 p-3 rounded-lg">
                        {manifest.manifest.release_notes}
                      </p>
                    </div>
                  )}

                  {/* Timestamps */}
                  <div className="mt-4 flex items-center gap-2 text-xs text-dark-500">
                    <HiOutlineClock size={14} />
                    <span>
                      Published: {new Date(manifest.manifest.timestamp).toLocaleString()}
                    </span>
                    {manifest.manifest.expires && (
                      <span>
                        • Expires: {new Date(manifest.manifest.expires).toLocaleString()}
                      </span>
                    )}
                  </div>
                </div>
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
};

export default UpdateCard;
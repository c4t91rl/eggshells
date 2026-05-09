import React from 'react';
import { useAppStore } from '../store/appStore';
import { api } from '../utils/tauriApi';
import UpdateCard from './UpdateCard';
import { HiOutlineArrowPath, HiOutlineInboxArrowDown } from 'react-icons/hi2';

const UpdateList: React.FC = () => {
  const {
    availableUpdates,
    setAvailableUpdates,
    isCheckingUpdates,
    setCheckingUpdates,
    addLog,
  } = useAppStore();

  const handleRefresh = async () => {
    setCheckingUpdates(true);
    try {
      const updates = await api.checkAllUpdates();
      setAvailableUpdates(updates);
      addLog('success', `Found ${updates.length} update(s)`);
    } catch (err) {
      addLog('error', `Failed to check updates: ${err}`);
    } finally {
      setCheckingUpdates(false);
    }
  };

  const verifiedUpdates = availableUpdates.filter((u) => u.verification.is_valid);
  const unverifiedUpdates = availableUpdates.filter((u) => !u.verification.is_valid);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-dark-50">Available Updates</h2>
          <p className="text-dark-400 text-sm mt-1">
            {availableUpdates.length} update(s) found across all servers
          </p>
        </div>
        <button
          onClick={handleRefresh}
          disabled={isCheckingUpdates}
          className="btn-secondary flex items-center gap-2"
        >
          <HiOutlineArrowPath
            size={18}
            className={isCheckingUpdates ? 'animate-spin' : ''}
          />
          Refresh
        </button>
      </div>

      {/* Verified Updates */}
      {verifiedUpdates.length > 0 && (
        <div>
          <h3 className="text-sm font-semibold text-green-400 mb-3 flex items-center gap-2">
            ✅ Verified Updates ({verifiedUpdates.length})
          </h3>
          <div className="space-y-3">
            {verifiedUpdates.map((update, idx) => (
              <UpdateCard key={idx} update={update} />
            ))}
          </div>
        </div>
      )}

      {/* Unverified Updates */}
      {unverifiedUpdates.length > 0 && (
        <div>
          <h3 className="text-sm font-semibold text-red-400 mb-3 flex items-center gap-2">
            ⚠️ Unverified Updates ({unverifiedUpdates.length})
          </h3>
          <div className="space-y-3">
            {unverifiedUpdates.map((update, idx) => (
              <UpdateCard key={idx} update={update} />
            ))}
          </div>
        </div>
      )}

      {availableUpdates.length === 0 && (
        <div className="glass-card p-12 text-center">
          <HiOutlineInboxArrowDown size={48} className="text-dark-600 mx-auto mb-4" />
          <h3 className="text-lg font-semibold text-dark-300">No Updates Available</h3>
          <p className="text-dark-500 text-sm mt-2">
            All software is up to date, or no servers are configured.
          </p>
        </div>
      )}
    </div>
  );
};

export default UpdateList;
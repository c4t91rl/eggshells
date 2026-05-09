"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
var react_1 = require("react");
var hi2_1 = require("react-icons/hi2");
var SettingsPanel = function () {
    var _a = (0, react_1.useState)(true), autoCheck = _a[0], setAutoCheck = _a[1];
    var _b = (0, react_1.useState)(true), requirePQ = _b[0], setRequirePQ = _b[1];
    var _c = (0, react_1.useState)(false), allowDowngrade = _c[0], setAllowDowngrade = _c[1];
    var _d = (0, react_1.useState)('3600'), checkInterval = _d[0], setCheckInterval = _d[1];
    return (<div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-dark-50">Settings</h2>
        <p className="text-dark-400 text-sm mt-1">
          Configure security preferences and update behavior
        </p>
      </div>

      {/* Update Settings */}
      <div className="glass-card p-6">
        <h3 className="section-title flex items-center gap-2 mb-4">
          <hi2_1.HiOutlineCog6Tooth size={20} className="text-primary-400"/>
          Update Preferences
        </h3>

        <div className="space-y-4">
          <div className="flex items-center justify-between p-4 bg-dark-800/50 rounded-xl">
            <div>
              <h4 className="font-medium text-dark-100">Auto-Check for Updates</h4>
              <p className="text-xs text-dark-400 mt-1">
                Periodically check registered servers for new updates
              </p>
            </div>
            <label className="relative inline-flex items-center cursor-pointer">
              <input type="checkbox" checked={autoCheck} onChange={function (e) { return setAutoCheck(e.target.checked); }} className="sr-only peer"/>
              <div className="w-11 h-6 bg-dark-600 rounded-full peer peer-checked:bg-primary-600 
                              peer-checked:after:translate-x-full after:content-[''] after:absolute 
                              after:top-0.5 after:left-[2px] after:bg-white after:rounded-full 
                              after:h-5 after:w-5 after:transition-all"/>
            </label>
          </div>

          <div className="flex items-center justify-between p-4 bg-dark-800/50 rounded-xl">
            <div>
              <h4 className="font-medium text-dark-100">Check Interval</h4>
              <p className="text-xs text-dark-400 mt-1">
                How often to check for updates (in seconds)
              </p>
            </div>
            <select value={checkInterval} onChange={function (e) { return setCheckInterval(e.target.value); }} className="bg-dark-700 border border-dark-600 rounded-lg px-3 py-1.5 text-sm text-dark-200">
              <option value="1800">Every 30 minutes</option>
              <option value="3600">Every hour</option>
              <option value="21600">Every 6 hours</option>
              <option value="86400">Daily</option>
            </select>
          </div>
        </div>
      </div>

      {/* Security Settings */}
      <div className="glass-card p-6">
        <h3 className="section-title flex items-center gap-2 mb-4">
          <hi2_1.HiOutlineShieldCheck size={20} className="text-quantum-400"/>
          Security Policy
        </h3>

        <div className="space-y-4">
          <div className="flex items-center justify-between p-4 bg-dark-800/50 rounded-xl">
            <div>
              <h4 className="font-medium text-dark-100">Require Post-Quantum Signatures</h4>
              <p className="text-xs text-dark-400 mt-1">
                Only accept updates signed with ML-DSA-65 or Hybrid algorithms
              </p>
            </div>
            <label className="relative inline-flex items-center cursor-pointer">
              <input type="checkbox" checked={requirePQ} onChange={function (e) { return setRequirePQ(e.target.checked); }} className="sr-only peer"/>
              <div className="w-11 h-6 bg-dark-600 rounded-full peer peer-checked:bg-quantum-600 
                              peer-checked:after:translate-x-full after:content-[''] after:absolute 
                              after:top-0.5 after:left-[2px] after:bg-white after:rounded-full 
                              after:h-5 after:w-5 after:transition-all"/>
            </label>
          </div>

          <div className="flex items-center justify-between p-4 bg-dark-800/50 rounded-xl">
            <div>
              <h4 className="font-medium text-dark-100">Allow Downgrade</h4>
              <p className="text-xs text-dark-400 mt-1">
                Allow installing older versions of packages (security risk)
              </p>
            </div>
            <label className="relative inline-flex items-center cursor-pointer">
              <input type="checkbox" checked={allowDowngrade} onChange={function (e) { return setAllowDowngrade(e.target.checked); }} className="sr-only peer"/>
              <div className="w-11 h-6 bg-dark-600 rounded-full peer peer-checked:bg-red-600 
                              peer-checked:after:translate-x-full after:content-[''] after:absolute 
                              after:top-0.5 after:left-[2px] after:bg-white after:rounded-full 
                              after:h-5 after:w-5 after:transition-all"/>
            </label>
          </div>
        </div>
      </div>

      {/* About */}
      <div className="glass-card p-6">
        <h3 className="section-title mb-4">About KryptoUpdate</h3>
        <div className="space-y-2 text-sm text-dark-400">
          <p><strong className="text-dark-200">Version:</strong> 0.1.0-alpha</p>
          <p><strong className="text-dark-200">Framework:</strong> Tauri 2 + React + Rust</p>
          <p><strong className="text-dark-200">PQ Library:</strong> pqcrypto-dilithium (Dilithium3 / ML-DSA-65)</p>
          <p><strong className="text-dark-200">Classical:</strong> ed25519-dalek</p>
          <p><strong className="text-dark-200">Hashing:</strong> SHA3 (sha3 crate) + BLAKE3</p>
          <p><strong className="text-dark-200">Platform:</strong> Linux / Windows (cross-platform)</p>
          <p className="mt-3 text-dark-500">
            Built for Kościuszkon 2026 – Honeywell Theme #1
          </p>
        </div>
      </div>
    </div>);
};
exports.default = SettingsPanel;

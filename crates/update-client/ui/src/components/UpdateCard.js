"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
var react_1 = require("react");
var VerificationBadge_1 = require("./VerificationBadge");
var hi2_1 = require("react-icons/hi2");
var framer_motion_1 = require("framer-motion");
var UpdateCard = function (_a) {
    var update = _a.update;
    var _b = (0, react_1.useState)(false), expanded = _b[0], setExpanded = _b[1];
    var manifest = update.manifest, verification = update.verification, publisher_name = update.publisher_name;
    var formatBytes = function (bytes) {
        if (bytes < 1024)
            return "".concat(bytes, " B");
        if (bytes < 1048576)
            return "".concat((bytes / 1024).toFixed(1), " KB");
        return "".concat((bytes / 1048576).toFixed(1), " MB");
    };
    var totalSize = manifest.manifest.files.reduce(function (sum, f) { return sum + f.size; }, 0);
    return (<div className="glass-card overflow-hidden">
      {/* Header */}
      <div className="p-5 cursor-pointer hover:bg-dark-800/30 transition-colors" onClick={function () { return setExpanded(!expanded); }}>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <div className={"w-12 h-12 rounded-xl flex items-center justify-center text-lg font-bold ".concat(verification.is_valid
            ? 'bg-green-500/15 text-green-400'
            : 'bg-red-500/15 text-red-400')}>
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
            {manifest.signatures.map(function (sig, idx) { return (<VerificationBadge_1.default key={idx} algorithm={sig.algorithm}/>); })}

            {verification.is_valid ? (<span className="status-ok">✓ Verified</span>) : (<span className="status-error">✗ Failed</span>)}

            <button className="btn-primary text-sm py-1.5 px-3">
              Install
            </button>

            {expanded ? (<hi2_1.HiOutlineChevronUp size={20} className="text-dark-400"/>) : (<hi2_1.HiOutlineChevronDown size={20} className="text-dark-400"/>)}
          </div>
        </div>
      </div>

      {/* Expanded Details */}
      <framer_motion_1.AnimatePresence>
        {expanded && (<framer_motion_1.motion.div initial={{ height: 0, opacity: 0 }} animate={{ height: 'auto', opacity: 1 }} exit={{ height: 0, opacity: 0 }} className="overflow-hidden">
            <div className="px-5 pb-5 border-t border-dark-700/50 pt-4">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                {/* Verification Details */}
                <div>
                  <h5 className="text-sm font-semibold text-dark-300 mb-3 flex items-center gap-2">
                    <hi2_1.HiOutlineFingerPrint size={16}/>
                    Verification Checks
                  </h5>
                  <div className="space-y-2">
                    {verification.checks.map(function (check, idx) { return (<div key={idx} className="flex items-start gap-2 text-sm bg-dark-800/50 p-2.5 rounded-lg">
                        <span className={check.passed ? 'text-green-400' : 'text-red-400'}>
                          {check.passed ? '✓' : '✗'}
                        </span>
                        <div className="flex-1">
                          <div className="text-dark-200 font-medium">{check.name}</div>
                          <div className="text-dark-500 text-xs mt-0.5">{check.details}</div>
                        </div>
                      </div>); })}
                  </div>

                  {verification.warnings.length > 0 && (<div className="mt-3">
                      <h6 className="text-xs font-semibold text-yellow-400 mb-1">Warnings</h6>
                      {verification.warnings.map(function (w, idx) { return (<p key={idx} className="text-xs text-yellow-400/70">{w}</p>); })}
                    </div>)}
                </div>

                {/* File Details */}
                <div>
                  <h5 className="text-sm font-semibold text-dark-300 mb-3 flex items-center gap-2">
                    <hi2_1.HiOutlineDocumentText size={16}/>
                    Files
                  </h5>
                  <div className="space-y-2">
                    {manifest.manifest.files.map(function (file, idx) { return (<div key={idx} className="bg-dark-800/50 p-3 rounded-lg">
                        <div className="flex justify-between text-sm">
                          <span className="text-dark-200 font-mono text-xs">{file.path}</span>
                          <span className="text-dark-400 text-xs">{formatBytes(file.size)}</span>
                        </div>
                        <div className="mt-1">
                          <code className="text-xs text-dark-500 font-mono break-all">
                            {file.hash_algorithm}: {file.hash.substring(0, 32)}...
                          </code>
                        </div>
                      </div>); })}
                  </div>

                  {/* Release Notes */}
                  {manifest.manifest.release_notes && (<div className="mt-4">
                      <h5 className="text-sm font-semibold text-dark-300 mb-2">Release Notes</h5>
                      <p className="text-sm text-dark-400 bg-dark-800/50 p-3 rounded-lg">
                        {manifest.manifest.release_notes}
                      </p>
                    </div>)}

                  {/* Timestamps */}
                  <div className="mt-4 flex items-center gap-2 text-xs text-dark-500">
                    <hi2_1.HiOutlineClock size={14}/>
                    <span>
                      Published: {new Date(manifest.manifest.timestamp).toLocaleString()}
                    </span>
                    {manifest.manifest.expires && (<span>
                        • Expires: {new Date(manifest.manifest.expires).toLocaleString()}
                      </span>)}
                  </div>
                </div>
              </div>
            </div>
          </framer_motion_1.motion.div>)}
      </framer_motion_1.AnimatePresence>
    </div>);
};
exports.default = UpdateCard;

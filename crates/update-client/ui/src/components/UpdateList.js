"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g = Object.create((typeof Iterator === "function" ? Iterator : Object).prototype);
    return g.next = verb(0), g["throw"] = verb(1), g["return"] = verb(2), typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (g && (g = 0, op[0] && (_ = 0)), _) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
Object.defineProperty(exports, "__esModule", { value: true });
var react_1 = require("react");
var appStore_1 = require("../store/appStore");
var tauriApi_1 = require("../utils/tauriApi");
var UpdateCard_1 = require("./UpdateCard");
var hi2_1 = require("react-icons/hi2");
var UpdateList = function () {
    var _a = (0, appStore_1.useAppStore)(), availableUpdates = _a.availableUpdates, setAvailableUpdates = _a.setAvailableUpdates, isCheckingUpdates = _a.isCheckingUpdates, setCheckingUpdates = _a.setCheckingUpdates, addLog = _a.addLog;
    var handleRefresh = function () { return __awaiter(void 0, void 0, void 0, function () {
        var updates, err_1;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    setCheckingUpdates(true);
                    _a.label = 1;
                case 1:
                    _a.trys.push([1, 3, 4, 5]);
                    return [4 /*yield*/, tauriApi_1.api.checkAllUpdates()];
                case 2:
                    updates = _a.sent();
                    setAvailableUpdates(updates);
                    addLog('success', "Found ".concat(updates.length, " update(s)"));
                    return [3 /*break*/, 5];
                case 3:
                    err_1 = _a.sent();
                    addLog('error', "Failed to check updates: ".concat(err_1));
                    return [3 /*break*/, 5];
                case 4:
                    setCheckingUpdates(false);
                    return [7 /*endfinally*/];
                case 5: return [2 /*return*/];
            }
        });
    }); };
    var verifiedUpdates = availableUpdates.filter(function (u) { return u.verification.is_valid; });
    var unverifiedUpdates = availableUpdates.filter(function (u) { return !u.verification.is_valid; });
    return (<div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-dark-50">Available Updates</h2>
          <p className="text-dark-400 text-sm mt-1">
            {availableUpdates.length} update(s) found across all servers
          </p>
        </div>
        <button onClick={handleRefresh} disabled={isCheckingUpdates} className="btn-secondary flex items-center gap-2">
          <hi2_1.HiOutlineArrowPath size={18} className={isCheckingUpdates ? 'animate-spin' : ''}/>
          Refresh
        </button>
      </div>

      {/* Verified Updates */}
      {verifiedUpdates.length > 0 && (<div>
          <h3 className="text-sm font-semibold text-green-400 mb-3 flex items-center gap-2">
            ✅ Verified Updates ({verifiedUpdates.length})
          </h3>
          <div className="space-y-3">
            {verifiedUpdates.map(function (update, idx) { return (<UpdateCard_1.default key={idx} update={update}/>); })}
          </div>
        </div>)}

      {/* Unverified Updates */}
      {unverifiedUpdates.length > 0 && (<div>
          <h3 className="text-sm font-semibold text-red-400 mb-3 flex items-center gap-2">
            ⚠️ Unverified Updates ({unverifiedUpdates.length})
          </h3>
          <div className="space-y-3">
            {unverifiedUpdates.map(function (update, idx) { return (<UpdateCard_1.default key={idx} update={update}/>); })}
          </div>
        </div>)}

      {availableUpdates.length === 0 && (<div className="glass-card p-12 text-center">
          <hi2_1.HiOutlineInboxArrowDown size={48} className="text-dark-600 mx-auto mb-4"/>
          <h3 className="text-lg font-semibold text-dark-300">No Updates Available</h3>
          <p className="text-dark-500 text-sm mt-2">
            All software is up to date, or no servers are configured.
          </p>
        </div>)}
    </div>);
};
exports.default = UpdateList;

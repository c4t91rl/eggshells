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
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
var jsx_runtime_1 = require("react/jsx-runtime");
var react_1 = require("react");
var appStore_1 = require("./store/appStore");
var tauriApi_1 = require("./utils/tauriApi");
var Layout_1 = __importDefault(require("./components/Layout"));
var Dashboard_1 = __importDefault(require("./components/Dashboard"));
var ServerManager_1 = __importDefault(require("./components/ServerManager"));
var UpdateList_1 = __importDefault(require("./components/UpdateList"));
var SecurityStatus_1 = __importDefault(require("./components/SecurityStatus"));
var SettingsPanel_1 = __importDefault(require("./components/SettingsPanel"));
var LogViewer_1 = __importDefault(require("./components/LogViewer"));
var App = function () {
    var _a = (0, appStore_1.useAppStore)(), currentPage = _a.currentPage, setServers = _a.setServers, addLog = _a.addLog, setSecurityInfo = _a.setSecurityInfo, setIntegrityReport = _a.setIntegrityReport;
    (0, react_1.useEffect)(function () {
        var init = function () { return __awaiter(void 0, void 0, void 0, function () {
            var servers, secInfo, report, err_1;
            return __generator(this, function (_a) {
                switch (_a.label) {
                    case 0:
                        _a.trys.push([0, 4, , 5]);
                        addLog('info', 'Initializing KryptoUpdate...');
                        return [4 /*yield*/, tauriApi_1.api.getServers()];
                    case 1:
                        servers = _a.sent();
                        setServers(servers);
                        addLog('info', "Loaded ".concat(servers.length, " registered server(s)"));
                        return [4 /*yield*/, tauriApi_1.api.getSecurityInfo()];
                    case 2:
                        secInfo = _a.sent();
                        setSecurityInfo(secInfo);
                        addLog('info', 'Security configuration loaded');
                        return [4 /*yield*/, tauriApi_1.api.getIntegrityReport()];
                    case 3:
                        report = _a.sent();
                        setIntegrityReport(report);
                        addLog(report.overall_status === 'Ok' ? 'success' : 'warn', "Integrity check: ".concat(report.overall_status));
                        addLog('success', 'KryptoUpdate initialized successfully');
                        return [3 /*break*/, 5];
                    case 4:
                        err_1 = _a.sent();
                        addLog('error', "Initialization failed: ".concat(err_1));
                        return [3 /*break*/, 5];
                    case 5: return [2 /*return*/];
                }
            });
        }); };
        init();
    }, []);
    var renderPage = function () {
        switch (currentPage) {
            case 'dashboard': return (0, jsx_runtime_1.jsx)(Dashboard_1.default, {});
            case 'servers': return (0, jsx_runtime_1.jsx)(ServerManager_1.default, {});
            case 'updates': return (0, jsx_runtime_1.jsx)(UpdateList_1.default, {});
            case 'security': return (0, jsx_runtime_1.jsx)(SecurityStatus_1.default, {});
            case 'settings': return (0, jsx_runtime_1.jsx)(SettingsPanel_1.default, {});
            case 'logs': return (0, jsx_runtime_1.jsx)(LogViewer_1.default, {});
            default: return (0, jsx_runtime_1.jsx)(Dashboard_1.default, {});
        }
    };
    return ((0, jsx_runtime_1.jsx)(Layout_1.default, { children: renderPage() }));
};
exports.default = App;

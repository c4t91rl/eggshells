"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
var jsx_runtime_1 = require("react/jsx-runtime");
var react_1 = require("react");
var appStore_1 = require("../store/appStore");
var hi2_1 = require("react-icons/hi2");
var LogViewer = function () {
    var _a = (0, appStore_1.useAppStore)(), logs = _a.logs, clearLogs = _a.clearLogs;
    var bottomRef = (0, react_1.useRef)(null);
    (0, react_1.useEffect)(function () {
        var _a;
        (_a = bottomRef.current) === null || _a === void 0 ? void 0 : _a.scrollIntoView({ behavior: 'smooth' });
    }, [logs]);
    var levelConfig = {
        info: { color: 'text-blue-400', label: 'INFO', icon: 'ℹ️' },
        warn: { color: 'text-yellow-400', label: 'WARN', icon: '⚠️' },
        error: { color: 'text-red-400', label: 'ERROR', icon: '❌' },
        success: { color: 'text-green-400', label: 'OK', icon: '✅' },
    };
    return ((0, jsx_runtime_1.jsxs)("div", { className: "space-y-6", children: [(0, jsx_runtime_1.jsxs)("div", { className: "flex items-center justify-between", children: [(0, jsx_runtime_1.jsxs)("div", { children: [(0, jsx_runtime_1.jsx)("h2", { className: "text-2xl font-bold text-dark-50", children: "Activity Log" }), (0, jsx_runtime_1.jsxs)("p", { className: "text-dark-400 text-sm mt-1", children: [logs.length, " entries \u2022 Real-time system activity"] })] }), (0, jsx_runtime_1.jsxs)("button", { onClick: clearLogs, className: "btn-danger flex items-center gap-2", children: [(0, jsx_runtime_1.jsx)(hi2_1.HiOutlineTrash, { size: 18 }), "Clear Logs"] })] }), (0, jsx_runtime_1.jsx)("div", { className: "glass-card p-4 max-h-[70vh] overflow-y-auto font-mono text-sm", children: logs.length === 0 ? ((0, jsx_runtime_1.jsx)("div", { className: "text-center text-dark-500 py-12", children: "No log entries yet" })) : ((0, jsx_runtime_1.jsxs)("div", { className: "space-y-1", children: [logs.map(function (log) {
                            var config = levelConfig[log.level];
                            return ((0, jsx_runtime_1.jsxs)("div", { className: "flex items-start gap-3 py-1.5 px-2 rounded hover:bg-dark-800/30", children: [(0, jsx_runtime_1.jsx)("span", { className: "text-dark-600 text-xs whitespace-nowrap", children: log.timestamp.toLocaleTimeString() }), (0, jsx_runtime_1.jsxs)("span", { className: "".concat(config.color, " text-xs font-bold w-12"), children: [config.icon, " ", config.label] }), (0, jsx_runtime_1.jsx)("span", { className: "text-dark-300 text-xs flex-1", children: log.message })] }, log.id));
                        }), (0, jsx_runtime_1.jsx)("div", { ref: bottomRef })] })) })] }));
};
exports.default = LogViewer;

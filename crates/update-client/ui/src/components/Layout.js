"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
var jsx_runtime_1 = require("react/jsx-runtime");
var Sidebar_1 = __importDefault(require("./Sidebar"));
var Layout = function (_a) {
    var children = _a.children;
    return ((0, jsx_runtime_1.jsxs)("div", { className: "flex h-screen w-screen overflow-hidden", children: [(0, jsx_runtime_1.jsx)(Sidebar_1.default, {}), (0, jsx_runtime_1.jsx)("main", { className: "flex-1 overflow-y-auto p-6", children: (0, jsx_runtime_1.jsx)("div", { className: "max-w-7xl mx-auto animate-fade-in", children: children }) })] }));
};
exports.default = Layout;

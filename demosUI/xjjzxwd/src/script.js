import React, { useState, useEffect } from "https://esm.sh/react@19";
import { createRoot } from "https://esm.sh/react-dom@19/client";
import { motion, AnimatePresence } from "https://esm.sh/motion/react";

const PHOTOS = [
{
  name: 'cafe',
  src: 'https://images.unsplash.com/photo-1773780413035-8891d3aee110?crop=entropy&cs=tinysrgb&fit=max&fm=jpg&ixid=M3wzMjM4NDZ8MHwxfHJhbmRvbXx8fHx8fHx8fDE3NzQ2NDQxOTZ8&ixlib=rb-4.1.0&q=80&w=400' },

{
  name: 'mountain',
  src: 'https://images.unsplash.com/photo-1773686044655-892102615409?crop=entropy&cs=tinysrgb&fit=max&fm=jpg&ixid=M3wzMjM4NDZ8MHwxfHJhbmRvbXx8fHx8fHx8fDE3NzQ2NDQyMTd8&ixlib=rb-4.1.0&q=80&w=400' },

{
  name: 'arch',
  src: 'https://images.unsplash.com/photo-1773158734206-eb5d39e37f3f?crop=entropy&cs=tinysrgb&fit=max&fm=jpg&ixid=M3wzMjM4NDZ8MHwxfHJhbmRvbXx8fHx8fHx8fDE3NzQ2NDQyMjV8&ixlib=rb-4.1.0&q=80&w=400' },

{
  name: 'railway',
  src: 'https://images.unsplash.com/photo-1558265996-a10bc3c4803e?crop=entropy&cs=tinysrgb&fit=max&fm=jpg&ixid=M3wzMjM4NDZ8MHwxfHJhbmRvbXx8fHx8fHx8fDE3NzQ2NDQyODN8&ixlib=rb-4.1.0&q=80&w=400' },

{
  name: 'hydrant',
  src: 'https://images.unsplash.com/photo-1729457046439-a8163a4ca95a?crop=entropy&cs=tinysrgb&fit=max&fm=jpg&ixid=M3wzMjM4NDZ8MHwxfHJhbmRvbXx8fHx8fHx8fDE3NzQ2NDQ0MjF8&ixlib=rb-4.1.0&q=80&w=400' },

{
  name: 'redcafe',
  src: 'https://images.unsplash.com/photo-1642582133866-a61bf1382bfe?crop=entropy&cs=tinysrgb&fit=max&fm=jpg&ixid=M3wzMjM4NDZ8MHwxfHJhbmRvbXx8fHx8fHx8fDE3NzQ2NDQ0NjB8&ixlib=rb-4.1.0&q=80&w=400' }];



const seededRandom = seed => {
  const x = Math.sin(seed) * 10000;
  return x - Math.floor(x);
};

const Polaroid = ({ src, name, style, transition = {} }) => {
  return /*#__PURE__*/(
    React.createElement(motion.div, {
      className: "aspect-4/5 shadow-sm bg-olive-100 cursor-pointer p-2",
      transition: transition
      // note: use inline style instead of tailwind css to prevent motion layout incorrect
      , style: style,
      layoutId: `photo-${name}` }, /*#__PURE__*/
    React.createElement("div", { className: "w-full aspect-square relative" }, /*#__PURE__*/
    React.createElement("img", {
      src: src,
      alt: name,
      className: "w-full h-full object-cover" }), /*#__PURE__*/

    React.createElement("div", { className: "absolute inset-0 shadow-[inset_0_8px_20px_rgba(0,0,0,0.15),inset_0_-8px_20px_rgba(0,0,0,0.15),inset_8px_0_20px_rgba(0,0,0,0.1),inset_-8px_0_20px_rgba(0,0,0,0.1)]" })), /*#__PURE__*/

    React.createElement("div", { className: "mt-2" }, name)));


};

const App = () => {
  const [collected, setCollected] = useState(false);

  return /*#__PURE__*/(
    React.createElement("div", { className: "flex flex-col gap-8 items-center py-8 select-none" }, /*#__PURE__*/

    React.createElement("div", {
      className: "grid grid-cols-3 gap-4 z-2",
      onClick: () => setCollected(true) },

    PHOTOS.map((p, i) => /*#__PURE__*/
    React.createElement(Polaroid, {
      key: p.name,
      src: p.src,
      name: p.name,
      style: { width: 120 },
      transition: { type: 'spring', bounce: 0.1, delay: i * 0.05 } }))), /*#__PURE__*/





    React.createElement("div", {
      className: `relative w-40 aspect-5/4 rounded-xl ${collected ? 'cursor-pointer transition-transform duration-250 hover:scale-105 transform-3d perspective-normal group' : ''}`,
      onClick: () => setCollected(false) }, /*#__PURE__*/
    React.createElement("div", { className: "absolute inset-0 bg-indigo-200 rounded-[inherit] shadow-md z-1" }, /*#__PURE__*/

    React.createElement("div", { className: "absolute inset-0 transition-transform group-hover:-translate-y-5" },
    collected && PHOTOS.map((p, i) => /*#__PURE__*/
    React.createElement(Polaroid, {
      key: p.name,
      src: p.src,
      name: p.name,
      transition: { type: 'spring', bounce: 0.2, delay: i * 0.05 },
      style: {
        rotate: seededRandom(i) * 20 - 10,
        width: 100,
        position: 'absolute', left: 0, right: 0, bottom: 24,
        margin: '0 auto' } })))), /*#__PURE__*/






    React.createElement("div", { className: "absolute left-0 bottom-0 w-full h-2/3 bg-white/20 backdrop-blur-sm rounded-[inherit] p-2 origin-bottom transition-transform group-hover:-rotate-x-15 group-hover:skew-x-5 z-3" }, /*#__PURE__*/
    React.createElement("div", { className: "text-sm text-white text-shadow-sm" }, "memories")))));





};

const root = createRoot(document.getElementById("app"));

root.render( /*#__PURE__*/React.createElement(App, null));
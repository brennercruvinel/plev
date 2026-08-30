/**
 * KAOSS!
 * @version 1.0.02
 * @summary 02-09-2021
 * @author Mads Stoumann
 * @description Web Audio KAOSS!
*/
function KAOSS(app) {
	const AudioContext = window.AudioContext || window.webkitAudioContext;
	const context = new AudioContext();
	const filter = context.createBiquadFilter();
	const gainNode = context.createGain();

	let frequency = 220;
	let oscillator;
	let playing = false;

	gainNode.connect(context.destination);
	gainNode.gain.value = 0.5;
	filter.connect(gainNode);

	app.addEventListener('xydown', down);
	app.addEventListener('xymove', e => move(e.detail.x, e.detail.y));
	app.addEventListener('xytoggle', toggle);
	app.addEventListener('xyup', up);
	app.elements.gain.addEventListener('input', () => gainNode.gain.value = app.elements.gain.valueAsNumber);
	app.elements.hue.addEventListener('input', () => app.style.setProperty('--hue', app.elements.hue.valueAsNumber));
	xyPad(app.elements.xypad, { scope:app, minY:27.5, maxY:440, y:220 });

	function down() {
		filter.type = app.elements.filter.value
		oscillator = context.createOscillator();
		oscillator.connect(filter);
		oscillator.frequency.value = frequency;
		oscillator.type = app.elements.wave.value;
		oscillator.start();
		playing = true;
	}

	function move(x, y) {
		if (playing) {
			frequency = y;
			oscillator.frequency.value = frequency;
			filter.frequency.value = frequency * 4;
			filter.Q.value = (x/4);
		}
	}

	function up() {
		if (playing) oscillator.stop(0);
		playing = false;
	}

	function toggle() {
		if (playing) {
			up();
		} else {
			down();
		}
	}
}

function xyPad(e,t){const n=Object.assign({leave:!0,minX:0,maxX:100,minY:0,maxY:100,scope:e,shift:10,x:50,y:50},t);let i,o,s,r,a,c,y,d,p,f=!1,l=n.x,u=n.y;const v=n.maxX-n.minX,x=n.maxY-n.minY,m=new ResizeObserver(()=>{y=e.getBoundingClientRect(),i=y.width,o=y.height,a=i/v,c=o/x,s=i-h,r=0-h,function(e,t){d=(e-n.minX)*a-h,p=(x-t+n.minY)*c-h}(l,u),g()}),h=e.elements.xyPoint.offsetWidth/2,w=()=>[parseInt(e.style.getPropertyValue("--tx")||0),parseInt(e.style.getPropertyValue("--ty")||0)];function g(e){var t,a;e&&(d=e.clientX-y.x-h,p=e.clientY-y.y-h),d>s&&(d=s),d<r&&(d=r),p>s&&(p=s),p<r&&(p=r),a=p,l=((t=d)+h)*(v/i)+n.minX,u=x-(a+h)*(x/o)+n.minY,b("--tx",t),b("--ty",a),E("xymove",{detail:{x:l,y:u,tx:t,ty:a}})}function E(e,t={}){n.scope.dispatchEvent(new CustomEvent(e,t))}function b(t,n,i=e){i.style.setProperty(t,n)}e.addEventListener("keydown",function(e){const t=e.shiftKey?n.shift*a:1*a,i=e.shiftKey?n.shift*c:1*c;let[o,s]=[...w()];"Tab"!==e.key&&e.preventDefault();switch(e.key){case"ArrowDown":s+=i;break;case"ArrowLeft":o-=t;break;case"ArrowRight":o+=t;break;case"ArrowUp":s-=i;break;case" ":E("xytoggle")}d=o,p=s,g()}),e.addEventListener("pointerdown",function(e){f=!0,E("xydown"),g(e)}),n.leave&&e.addEventListener("pointerleave",()=>e.dispatchEvent(new Event("pointerup"))),e.addEventListener("pointermove",function(e){f&&g(e)}),e.addEventListener("pointerup",function(){f=!1,E("xyup")}),m.observe(e)};

/* INIT */
const elm = document.querySelector(`[data-blok="kaoss"]`);
KAOSS(elm);
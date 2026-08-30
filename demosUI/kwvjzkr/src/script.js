let swiper = new Swiper(".steps", {
	slidesPerView: 1,
	spaceBetween: 20,
	autoHeight: true,
	pagination: {
		el: ".swiper-pagination",
		clickable: true
	},
	breakpoints: {
		768: {
			slidesPerView: 2
		},
		1024: {
			slidesPerView: 3
		}
	}
});

// Envolve tabelas do conteudo em um wrapper rolavel para evitar que
// tabelas largas estourem ou fiquem espremidas em telas pequenas.
document.addEventListener("DOMContentLoaded", function () {
	const tables = document.querySelectorAll("article table");

	tables.forEach((table) => {
		// Ignora tabelas de realce de codigo (numeros de linha dentro de <pre>).
		if (table.closest("pre")) return;
		// Evita envolver duas vezes.
		if (table.parentElement && table.parentElement.classList.contains("table-scroll")) return;

		const wrapper = document.createElement("div");
		wrapper.className = "table-scroll";
		// Permite foco por teclado para rolar a tabela com as setas.
		wrapper.setAttribute("tabindex", "0");
		wrapper.setAttribute("role", "region");
		wrapper.setAttribute("aria-label", "Tabela rolavel");

		table.parentNode.insertBefore(wrapper, table);
		wrapper.appendChild(table);
	});
});
